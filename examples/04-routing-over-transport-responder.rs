//! This node starts a ble listener and an echoer worker.  It then
//! runs forever waiting for messages.

#![allow(
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables,
)]

// - features -----------------------------------------------------------------

#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

// log transport
#[cfg(any(feature = "stm32f4", feature = "stm32h7"))]
use panic_itm as _;
#[cfg(feature = "atsame54")]
use panic_semihosting as _;

// atsame54
#[cfg(feature = "atsame54")]
extern crate atsame54_xpro as hal;
#[cfg(feature = "atsame54")]
use embedded_hal;

// stm32f4
#[cfg(feature = "stm32f4")]
use stm32f4xx_hal as hal;
#[cfg(feature = "stm32f4")]
use stm32f4xx_hal::hal as embedded_hal;

// daisy
#[cfg(feature = "bsp_daisy")]
use daisy_bsp as bsp;
#[cfg(feature = "bsp_daisy")]
use daisy_bsp::hal;
#[cfg(feature = "bsp_daisy")]
use daisy_bsp::hal::hal as embedded_hal;

// nucleo-h7xx
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx as bsp;
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx::hal as hal;
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx::hal::hal as embedded_hal;

// logging macros
#[cfg_attr(not(feature = "std"), macro_use)]
extern crate ockam_executor;

// hal version mismatch
#[cfg(not(feature = "atsame54"))]
use hal::time::NanoSeconds;
#[cfg(not(feature = "atsame54"))]
use hal::time::MilliSeconds;
#[cfg(feature = "atsame54")]
use hal::time::Nanoseconds as NanoSeconds;
#[cfg(feature = "atsame54")]
use hal::time::Milliseconds as MilliSeconds;


// - dependencies -------------------------------------------------------------

use hal::prelude::*;
use hal::pac;

use embedded_hal::spi;

use ockam::println;


// - modules ------------------------------------------------------------------

mod allocator;


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


fn main() -> core::result::Result<(), u32> {

    // - initialize allocator -------------------------------------------------

    allocator::init();
    allocator::stats(0);

    // - ockam::node ----------------------------------------------------------

    use hello_ockam::Echoer;
    use ockam::{Context, Result};
    use ockam_transport_ble::BleTransport;

    #[ockam::node]
    async fn async_main(ctx: Context) -> Result<()> {

        // - configure board --------------------------------------------------

        #[cfg(feature = "atsame54")]
        let (mut timer, spi, spi_nss, spi_irq, spi_rst) = {
            use hal::clock::GenericClockController;
            use hal::pac::{CorePeripherals, Peripherals};
            use hal::prelude::*;
            use hal::timer::TimerCounter;
            use hal::watchdog::{Watchdog, WatchdogTimeout};

            let cp = pac::CorePeripherals::take().unwrap();
            let mut peripherals = pac::Peripherals::take().unwrap();

            let mut clocks = GenericClockController::with_internal_32kosc(
                peripherals.GCLK,
                &mut peripherals.MCLK,
                &mut peripherals.OSC32KCTRL,
                &mut peripherals.OSCCTRL,
                &mut peripherals.NVMCTRL,
            );

            let gclk0 = clocks.gclk0();
            let tc45 = clocks.tc4_tc5(&gclk0).unwrap();
            let mut timer = TimerCounter::tc4_(&tc45, peripherals.TC4, &mut peripherals.MCLK);

            let mut pins = hal::Pins::new(peripherals.PORT);

            // - configure spi interface for STEVAL-IDB005V1D -----------------

            let spi6_irq = pins.pd00.into_pull_down_input(&mut pins.port);
            let spi6_csn = pins.spi6_ss.into_push_pull_output(&mut pins.port);
            let spi6_rst = pins.pb01.into_push_pull_output(&mut pins.port);

            let mut spi6 = hal::pins::SPI {
                sck: pins.sck,
                mosi: pins.mosi,   // sdi
                miso: pins.miso,   // sdo
            }.init(
                &mut clocks,
                1.mhz(),
                peripherals.SERCOM6,
                &mut peripherals.MCLK,
                &mut pins.port,
            );

            (timer, spi6, spi6_csn, spi6_irq, spi6_rst)
        };

        #[cfg(feature = "stm32h7")]
        let (mut timer, spi, spi_nss, spi_irq, spi_rst) = {
            use hal::timer::Timer;
            use hello_ockam::boards;

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

            let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                         dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                         dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                         dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                         dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                         dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                         dp.GPIOG.split(ccdr.peripheral.GPIOG));

            // - configure spi interface for STEVAL-IDB005V1D -----------------

            let mut timer = dp.TIM7.timer(1.hz(), ccdr.peripheral.TIM7, &ccdr.clocks);

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

            (timer, spi3, spi3_nss, spi3_irq, spi3_rst)
        };

        println!("Hello ockam_transport_ble!");

        // - bluenrg ----------------------------------------------------------

        static mut RX_BUFFER: [u8; 1024] = [0; 1024];
        let mut bluenrg = bluenrg::BlueNRG::new(
            unsafe { &mut RX_BUFFER },
            spi_nss,
            spi_irq,
            spi_rst
        );

        // - ockam::driver ----------------------------------------------------

        use ockam_transport_ble::driver::bluetooth_hci::BleAdapter;
        use ockam_transport_ble::driver::BleServer;

        let mut ble_adapter = BleAdapter::with_interface(spi, bluenrg);
        ble_adapter.reset(&mut timer, MilliSeconds(50).into())?;

        let ble_server = BleServer::with_adapter(ble_adapter);

        // - the actual example! ----------------------------------------------

        // Initialize the BLE Transport.
        println!("[main] Initialize the BLE Transport.");
        let ble = BleTransport::create(&ctx).await?;

        // Create a BLE listener and wait for incoming connections.
        println!("[main] Create a BLE listener and wait for incoming connections.");
        ble.listen(ble_server, "ockam_ble_1").await?;

        // Create an echoer worker
        println!("[main] Create an echoer worker");
        ctx.start_worker("echoer", Echoer).await?;

        // Don't call ctx.stop() here so this node runs forever.
        println!("[main] run forever");
        Ok(())
    }

    // - main loop ------------------------------------------------------------

    println!("entering main loop");
    loop {
        cortex_m::asm::wfi();
    }
}
