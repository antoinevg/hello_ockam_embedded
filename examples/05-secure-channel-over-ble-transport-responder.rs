//! This node starts a ble listener and an echoer worker.  It then
//! runs forever waiting for messages.

// - features -----------------------------------------------------------------

#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

// log transport
#[cfg(any(feature = "stm32f4", feature = "stm32h7"))]
use panic_itm as _;
#[cfg(feature = "atsame54")]
use panic_halt as _;

// atsame54
#[cfg(feature = "atsame54")]
extern crate atsame54_xpro as hal;

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

// hal mismatches
#[cfg(any(feature = "stm32h7", feature = "pic32"))]
use hal::time::MilliSeconds;
#[cfg(feature = "stm32f4")]
use stm32f4xx_hal_fugit::MillisDurationU32 as MilliSeconds;
#[cfg(feature = "atsame54")]
use atsame54_xpro::time::Milliseconds as MilliSeconds;


// - dependencies -------------------------------------------------------------

use hello_ockam::allocator;
use hello_ockam::tracing_subscriber;

use ockam_core::println;

use hal::pac;


// - entry --------------------------------------------------------------------

#[cortex_m_rt::entry]
fn entry() -> ! {
    match sync_main() {
        Ok(_) => (),
        Err(e) => {
            println!("main error: {:?}", e);
        }
    }
    loop {}
}


fn sync_main() -> ockam::Result<()> {

    // - initialize allocator -------------------------------------------------

    allocator::init();
    allocator::stats(0);

    // - register tracing subscriber ------------------------------------------

    #[cfg(feature="log-semihosting")]
    tracing_subscriber::register();

    // - ockam::node ----------------------------------------------------------

    use ockam::compat::{boxed::Box, string::String};

    use ockam::authenticated_storage::InMemoryStorage;
    use ockam::identity::{Identity, TrustEveryonePolicy};
    use ockam::{vault::Vault, Context, Result, Routed, Worker};

    use ockam_transport_ble::BleTransport;

    #[ockam::node(no_main)]
    async fn main(ctx: Context) -> Result<()> {

        // - configure board --------------------------------------------------

        #[cfg(feature = "atsame54")]
        let (mut timer, spi, spi_nss, spi_irq, spi_rst) = {
            use hal::clock::GenericClockController;
            use hal::prelude::*;
            use hal::timer::TimerCounter;

            let mut dp = pac::Peripherals::take().unwrap();

            let mut clocks = GenericClockController::with_internal_32kosc(
                dp.GCLK,
                &mut dp.MCLK,
                &mut dp.OSC32KCTRL,
                &mut dp.OSCCTRL,
                &mut dp.NVMCTRL,
            );

            let gclk0 = clocks.gclk0();
            let tc45 = clocks.tc4_tc5(&gclk0).unwrap();
            let timer = TimerCounter::tc4_(&tc45, dp.TC4, &mut dp.MCLK);

            let mut pins = hal::Pins::new(dp.PORT);

            // - configure & register uart for tracing ------------------------

            #[cfg(feature="log-uart")]
            {
                use hal::sercom::v2::uart;

                let pads = uart::Pads::default()
                    .rx(pins.uart0_rx)   // pa05
                    .tx(pins.uart0_tx);  // pa04
                let gclk0 = clocks.gclk0();
                let clock = &clocks.sercom0_core(&gclk0).unwrap();
                let config = uart::Config::new(
                    &dp.MCLK,
                    dp.SERCOM0,
                    pads,
                    clock.freq()
                );
                let uart5 = config
                    .baud(115_200.mhz(), uart::BaudMode::Arithmetic(uart::Oversampling::Bits8))
                    .char_size::<uart::EightBit>()
                    .parity(uart::Parity::None)
                    .stop_bits(uart::StopBits::OneBit)
                    .enable();
                tracing_subscriber::register_with_uart(uart5);
            }

            // - configure spi interface for STEVAL-IDB005V1D -----------------

            let (spi6_irq, spi6_csn, spi6_rst) = {
                // looks like we're stuck on spi v1 until bluenrg upgrades
                #[allow(deprecated)]
                (pins.pd00.into_pull_down_input(&mut pins.port),
                 pins.spi6_ss.into_push_pull_output(&mut pins.port),
                 pins.pb01.into_push_pull_output(&mut pins.port))
            };

            let spi6 = hal::pins::SPI {
                sck: pins.sck,
                mosi: pins.mosi,   // sdi
                miso: pins.miso,   // sdo
            }.init(
                &mut clocks,
                1.mhz(),
                dp.SERCOM6,
                &mut dp.MCLK,
                &mut pins.port,
            );

            (timer, spi6, spi6_csn, spi6_irq, spi6_rst)
        };

        #[cfg(feature = "stm32h7")]
        let (mut timer, spi, spi_nss, spi_irq, spi_rst) = {
            use embedded_hal::spi;
            use embedded_hal::digital::v2::OutputPin;
            use hal::prelude::*;

            let board = unsafe { bsp::Board::steal() };
            let dp = pac::Peripherals::take().unwrap();
            let ccdr = board.freeze_clocks_with(
                dp.PWR.constrain(),
                dp.RCC.constrain(),
                &dp.SYSCFG,
                |pwrcfg, rcc, syscfg| {
                    rcc.sys_ck(96.MHz())                // system clock @ 96 MHz
                        // pll1 drives system clock
                        .pll1_strategy(hal::rcc::PllConfigStrategy::Iterative)
                        .pll1_r_ck(96.MHz())             // TRACECLK
                        .pll1_q_ck(48.MHz())             // spi clock
                        .pll3_p_ck((48_000 * 256).Hz())  // sai clock @ 12.288 MHz
                        .freeze(pwrcfg, syscfg)
                }
            );

            #[cfg(feature="log-itm")]
            tracing_subscriber::register();

            let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                         dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                         dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                         dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                         dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                         dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                         dp.GPIOG.split(ccdr.peripheral.GPIOG));

            // - configure spi interface for STEVAL-IDB005V1D -----------------

            let timer = dp.TIM7.timer(1.Hz(), ccdr.peripheral.TIM7, &ccdr.clocks);

            let spi3_irq  = pins.d43.into_pull_down_input();
            let spi3_rst  = pins.d44.into_push_pull_output();
            let spi3_sck  = pins.d45.into_alternate().speed(hal::gpio::Speed::VeryHigh);
            let spi3_miso = pins.d46.into_alternate().speed(hal::gpio::Speed::VeryHigh);
            let spi3_mosi = pins.d47.into_alternate().speed(hal::gpio::Speed::VeryHigh);

            let mut spi3_nss  = pins.d20.into_push_pull_output();
            spi3_nss.set_high();

            let config = hal::spi::Config::new(
                spi::Mode {
                    polarity: spi::Polarity::IdleLow,
                    phase: spi::Phase::CaptureOnFirstTransition,
                }
            );

            let spi3 = dp.SPI3.spi(
                (spi3_sck, spi3_miso, spi3_mosi),
                config,
                3.MHz(),
                ccdr.peripheral.SPI3,
                &ccdr.clocks,
            );

            (timer, spi3, spi3_nss, spi3_irq, spi3_rst)
        };

        println!("Hello ockam_transport_ble!");

        // - bluenrg ----------------------------------------------------------

        static mut RX_BUFFER: [u8; 1024] = [0; 1024];
        let bluenrg = bluenrg::BlueNRG::new(
            unsafe { &mut RX_BUFFER },
            spi_nss,
            spi_irq,
            spi_rst
        );

        // - ockam::driver ----------------------------------------------------

        use ockam_transport_ble::driver::bluetooth_hci::BleAdapter;
        use ockam_transport_ble::driver::BleServer;

        let mut ble_adapter = BleAdapter::with_interface(spi, bluenrg);
        #[cfg(not(feature="stm32h7"))]
        ble_adapter.reset(&mut timer, MilliSeconds(50).into())?;
        // TODO !!!!!!!!! #[cfg(feature="stm32h7")]
        //ble_adapter.reset(&mut timer, MilliSeconds::from_ticks(50).into())?;

        let ble_server = BleServer::with_adapter(ble_adapter);

        // - echoer -----------------------------------------------------------

        // Define an Echoer worker that prints any message it receives and
        // echoes it back on its return route.
        #[ockam::worker]
        impl Worker for Echoer {
            type Context = Context;
            type Message = String;

            async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<String>) -> Result<()> {
                println!("\n[echoer] [✓] Address: {}, Received: {}", ctx.address(), msg);

                // Echo the message body back on its return_route.
                println!("[echoer] [✓] Echoing message '{}' back on its return_route: {}",
                         &msg,
                         msg.return_route());

                ctx.send(msg.return_route(), msg.body()).await
            }
        }
        struct Echoer;

        // - example code -----------------------------------------------------

        // Create an echoer worker
        println!("[main] Create an echoer worker");
        ctx.start_worker("echoer", Echoer).await?;

        // Initialize the BLE Transport.
        println!("[main] Initialize the BLE Transport.");
        let ble = BleTransport::create(&ctx).await?;

        // Create a Vault to safely store secret keys for Bob.
        println!("[main] Create a Vault to safely store secret keys for Bob.");
        let vault = Vault::create();

        // Create an Identity to represent Bob.
        println!("[main] Create an Identity to represent Bob.");
        let bob = Identity::create(&ctx, &vault).await?;

        // Create an AuthenticatedStorage to store info about Bob's known Identities.
        let storage = InMemoryStorage::new();

        // Create a BLE listener and wait for incoming connections.
        println!("[main] Create a BLE listener that will wait for incoming connections.");
        ble.listen(ble_server, "ockam_ble_1").await?;

        // Create a secure channel listener for Bob that will wait for requests to
        // initiate an Authenticated Key Exchange.
        println!("[main] Create a secure channel listener for Bob.");
        bob.create_secure_channel_listener("bob_listener", TrustEveryonePolicy, &storage)
            .await?;

        // Don't call ctx.stop() here so this node runs forever.
        println!("[main] run forever");

        Ok(()) as ockam::Result<()>
    }


    // - main loop ------------------------------------------------------------

    println!("entering main loop");
    loop {
        cortex_m::asm::wfi();
    }
}
