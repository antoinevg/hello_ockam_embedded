#![allow(
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables,
)]

#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[cfg_attr(not(feature = "std"), macro_use)]
extern crate ockam_executor;

use ockam_transport_ble::BleAddr;
use panic_itm as _;

#[cfg(feature = "stm32f4")]
use stm32f4xx_hal as hal;
#[cfg(feature = "bsp_daisy")]
use daisy_bsp as bsp;
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx as bsp;
use bsp::{hal, pac, hal as embedded_hal};

use bluenrg::gatt::Commands;
use bluetooth_hci::host::uart::Hci as _;
use core2::io::{Cursor, Write};
use core::result::Result;
use embedded_hal::spi;
use hal::delay::Delay;
use hal::nb::block;
use hal::prelude::*;
use hal::timer::Timer;

use hello_ockam::boards;
use ockam_transport_ble::driver::bluetooth_hci::ble;
use ockam_transport_ble::driver::bluetooth_hci::ble_uart;

// - println ------------------------------------------------------------------

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        let itm = unsafe { &mut *cortex_m::peripheral::ITM::ptr() };
        cortex_m::iprintln!(&mut itm.stim[0], $($arg)*);
    }};
}


// - modules ------------------------------------------------------------------

mod allocator;

//pub mod ble;
//pub mod ble_uart;


// - entry --------------------------------------------------------------------

#[cortex_m_rt::entry]
fn entry() -> ! {
    match main() {
        Ok(_) => (),
        Err(e) => {
            println!("main error: {:?}", e);
        }
    }

    loop {}
}


fn main() -> Result<(), u32> {
    // - initialize allocator -------------------------------------------------

    allocator::init();


    // - configure board ------------------------------------------------------

    let board = unsafe { bsp::Board::steal() };
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let ccdr = boards::freeze_clocks_with_config(
        dp.PWR.constrain(), dp.RCC.constrain(), &dp.SYSCFG,
        |pwrcfg, rcc, syscfg| {
            rcc.sys_ck(96.mhz())                // system clock @ 96 MHz
            // pll1 drives system clock
                .pll1_strategy(hal::rcc::PllConfigStrategy::Iterative)
                .pll1_r_ck(96.mhz())             // TRACECLK
                .pll1_q_ck(48.mhz())             // spi clock
                .pll3_p_ck((48_000 * 256).hz())  // sai clock @ 12.288 MHz
                .freeze(pwrcfg, syscfg)
        }
    );

    println!("Hello ockam_transport_ble!");

    let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                 dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                 dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                 dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                 dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                 dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                 dp.GPIOG.split(ccdr.peripheral.GPIOG));

    let mut user_leds = bsp::led::UserLeds::new(pins.user_leds);


    // - configure spi --------------------------------------------------------

    let mut timer = dp.TIM7.timer(1.hz(), ccdr.peripheral.TIM7, &ccdr.clocks);
    let mut delay = Delay::new(cp.SYST, ccdr.clocks);

    let spi3_irq  = pins.d43.into_pull_down_input();
    let spi3_rst  = pins.d44.into_push_pull_output();
    let spi3_sck  = pins.d45.into_alternate_af6().set_speed(hal::gpio::Speed::VeryHigh);
    let spi3_miso = pins.d46.into_alternate_af6().set_speed(hal::gpio::Speed::VeryHigh);
    let spi3_mosi = pins.d47.into_alternate_af6().set_speed(hal::gpio::Speed::VeryHigh);

    use nucleo_h7xx::embedded_hal::digital::v2::OutputPin;
    let mut spi3_nss  = pins.d20.into_push_pull_output();
    spi3_nss.set_high().ok();

    let config = hal::spi::Config::new(
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        }
    );

    let mut spi3 = dp.SPI3.spi(
        (spi3_sck, spi3_miso, spi3_mosi),
        config,
        3.mhz(),
        ccdr.peripheral.SPI3,
        &ccdr.clocks,
    );


    // - bluenrg --------------------------------------------------------------

    use ockam_transport_ble::driver::CHARACTERISTIC_VALUE_LENGTH;
    let ble_addr = ockam_transport_ble::parse_ble_addr("ockam_ble_1").unwrap();

    let mut rx_buffer: [u8; CHARACTERISTIC_VALUE_LENGTH] = [0; CHARACTERISTIC_VALUE_LENGTH];
    let mut bluetooth = bluenrg::BlueNRG::new(
        &mut rx_buffer,
        spi3_nss,
        spi3_irq,
        spi3_rst
    );

    // hardware reset
    println!("\n\treset bluenrg-ms device");
    bluetooth.reset(&mut timer, 2.hz()).ok();
    match bluetooth.with_spi(&mut spi3, |controller| block!(controller.read())) {
        Ok(packet) => {
            let bluetooth_hci::host::uart::Packet::Event(event) = packet;
            ble_uart::dispatch_event(&event);
        }
        Err(e) => println!("reset error: {:?}", e),
    }

    // test device comms
    ble::read_local_version_information(&mut spi3, &mut bluetooth)
        .expect("ble::read_local_version_information failed");


    // - configure ble uart ---------------------------------------------------

    ble_uart::setup(&mut spi3, &mut bluetooth)
        .expect("ble_uart::setup failed");
    delay.delay_ms(500u16);
    let mut context = ble_uart::initialize_gatt_and_gap(&mut spi3, &mut bluetooth, &ble_addr)
        .expect("ble_uart::initialize_gatt_and_gap failed");
    delay.delay_ms(500u16);
    ble_uart::initialize_uart(&mut spi3, &mut bluetooth, &mut context)
        .expect("ble_uart::initialize_uart failed");
    delay.delay_ms(500u16);


    // - main loop ------------------------------------------------------------

    let mut counter: usize = 0;

    println!("\nentering main loop");

    #[derive(Debug, PartialEq)]
    enum State {
        Disconnected,
        Advertising,
        Connected,
        Error,
    }

    use ockam_transport_ble::driver::bluetooth_hci::Event;

    //use bluetooth_hci::event::command::ReturnParameters::Vendor as Vendor;
    //use bluetooth_hci::Vendor;

    use bluenrg::event::BlueNRGEvent;

    let mut state = State::Disconnected;
    loop {
        match state {
            State::Disconnected => {
                ble_uart::start_advertising(&mut spi3, &mut bluetooth, &mut context, &ble_addr)
                    .expect("ble_uart::make_connection failed");
                state = State::Advertising;
                println!("\nstate = {:?}", state);
            }
            State::Advertising => {
            }
            State::Connected => {
            }
            State::Error => {
            }
        }

        ble_uart::poll(&mut spi3, &mut bluetooth, |event| {
            match event {
                Event::LeConnectionComplete(event) => {
                    println!("\t=> LeConnectionComplete -> {:?}", event);
                    state = State::Connected;
                    println!("\nstate = {:?}", state);
                }
                Event::Vendor(BlueNRGEvent::GattAttributeModified(event)) => {
                    if event.attr_handle == context.uart_rx_attribute_handle
                        .expect("rx attribute handle is not set")
                    {
                        if let Ok(data) = core::str::from_utf8(event.data()) {
                            println!("\t=> Rx: -> {:?}", data);
                        } else {
                            println!("\t=> Rx: -> {:?}", event.data());
                        }
                    } else {
                        println!("\t=> Rx unknown: -> {:?}", event);
                    }
                }
                Event::DisconnectionComplete(event) => {
                    println!("\t=> DisconnectionComplete -> {:?}", event);
                    state = State::Disconnected;
                    println!("\nstate = {:?}", state);
                }
                _ => {
                    println!("\t=> unknown event: {:?}", event);
                }
            }
            Some(State::Error)
        });

        if counter % 1000_000 == 0 && state == State::Connected {
            //led.toggle().ok();

            // create message buffer
            let mut tx_buffer = [0 as u8; CHARACTERISTIC_VALUE_LENGTH];
            let mut tx_cursor = Cursor::new(&mut tx_buffer[..]);
            match write!(&mut tx_cursor, "server counter: {}", counter / 100_000) {
            //match write!(&mut tx_cursor, "0123456789012345678 server: {}", counter / 100_000) {
                Ok(()) => (),
                Err(e) => {
                    println!("failed write: {:?}", e);
                    continue
                }
            }
            let position: usize = tx_cursor.position() as usize;
            let tx_buffer = &tx_cursor.into_inner()[0..position];

            // send message buffer
            block!(bluetooth.with_spi(&mut spi3, |controller| {
                controller.update_characteristic_value(&bluenrg::gatt::UpdateCharacteristicValueParameters {
                    service_handle: context.uart_service_handle.expect("uart service handle has not been set"),
                    characteristic_handle: context.uart_tx_handle.expect("uart tx handle has not been set"),
                    offset: 0x00,
                    value: tx_buffer,
                })
            })).unwrap();
            ble_uart::controller_read(&mut spi3, &mut bluetooth);
        }

        if counter >= 0xffff_fffe {
            break;
        } else {
            counter += 1;
        }
    }

    Ok(())
}
