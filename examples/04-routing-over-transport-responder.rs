//! This node starts a ble listener and an echoer worker.  It then
//! runs forever waiting for messages.

#![allow(
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables,
)]

#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

use panic_itm as _;

#[cfg(feature = "stm32f4")]
use stm32f4xx_hal as hal;
#[cfg(feature = "bsp_daisy")]
use daisy_bsp as bsp;
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx as bsp;

use bsp::hal;
//use bsp::loggit as println;

use hal::prelude::*;
use hal::time::Hertz;
use hal::delay::Delay;
use hal::timer::Timer;
use hal::time::U32Ext;
use hal::pac;

use hal::hal as embedded_hal;
use embedded_hal::spi;

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

    // - ockam::node ----------------------------------------------------------

    use hello_ockam::Echoer;
    use ockam::{Context, Result};
    use ockam_transport_ble::BleTransport;

    #[ockam::node]
    async fn async_main(ctx: Context) -> Result<()> {

        // - configure board ------------------------------------------------------

        let board = unsafe { bsp::Board::steal() };
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();
        //let ccdr = board.freeze_clocks(dp.PWR.constrain(), dp.RCC.constrain(), &dp.SYSCFG);

        let ccdr = {
            // SAI clock uses pll3
            const PLL3_P: Hertz = Hertz(48_000 * 256);

            let (pwr, rcc, syscfg) = (dp.PWR.constrain(),
                                      dp.RCC.constrain(),
                                      &dp.SYSCFG);

            let mut cp = unsafe { cortex_m::Peripherals::steal() };
            let dp = unsafe { pac::Peripherals::steal() };

            // link SRAM3 power state to CPU1
            dp.RCC.ahb2enr.modify(|_, w| w.sram3en().set_bit());

            //let pwrcfg = pwr.smps().vos0(syscfg).freeze();
            let pwrcfg = pwr.freeze();

            let ccdr = rcc
                .sys_ck(200.mhz())                      // system clock @ 480 MHz
                .pll1_strategy(hal::rcc::PllConfigStrategy::Iterative) // pll1 drives system clock
                .pll1_r_ck(200.mhz())                   // TRACECLK
                .pll1_q_ck(200.mhz())
                //.pll3_p_ck(PLL3_P)                    // sai clock @ 12.288 MHz
                .freeze(pwrcfg, syscfg);

            unsafe {
                let swo_frequency = 2_000_000;
                bsp::itm::enable_itm(&mut cp.DCB,
                                     &dp.DBGMCU,
                                     &mut cp.ITM,
                                     ccdr.clocks.c_ck().0,
                                     swo_frequency);
            }

            // configure cpu
            cp.SCB.invalidate_icache();
            cp.SCB.enable_icache();
            cp.DWT.enable_cycle_counter();

            ccdr
        };

        let mut delay = Delay::new(cp.SYST, ccdr.clocks);
        delay.delay_ms(200u16);

        println!("Hello ockam_transport_ble!");

        delay.delay_ms(200u16);

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

        // - bluenrg ----------------------------------------------------------

        //#[link_section = ".axisram.buffers"]
        static mut RX_BUFFER: [u8; 1024] = [0; 1024]; // TODO how to calculate
        let mut bluetooth = bluenrg::BlueNRG::new(
            unsafe { &mut RX_BUFFER }, // TODO
            spi3_nss,
            spi3_irq,
            spi3_rst
        );

        // - ockam::driver ----------------------------------------------------

        use ockam_transport_ble::driver::bluetooth_hci::BleAdapter;
        use ockam_transport_ble::driver::BleServer;

        let mut ble_adapter = BleAdapter::with_interface(spi3, bluetooth);
        ble_adapter.reset(&mut timer)?;

        let ble_server = BleServer::with_adapter(ble_adapter);


        // - the actual example! ----------------------------------------------

        // Create an echoer worker - TODO race condition if we create this after BleTransport ?
        println!("[main] Create an echoer worker");
        ctx.start_worker("echoer", Echoer).await?;

        // Initialize the BLE Transport.
        println!("[main] Initialize the BLE Transport.");
        let ble = BleTransport::create(&ctx).await?;

        // Create a BLE listener and wait for incoming connections.
        println!("[main] Create a BLE listener and wait for incoming connections.");
        ble.listen(ble_server, "ockam_ble_1").await?;

        // Don't call ctx.stop() here so this node runs forever.
        println!("[main] run forever");
        Ok(())
    }

    //println!("main 3");

    // - main loop ------------------------------------------------------------

    //println!("entering main loop");
    loop {
        cortex_m::asm::wfi();
    }
}
