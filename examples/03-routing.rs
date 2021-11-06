#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

// - bare metal dependencies --------------------------------------------------

#[cfg(not(feature = "std"))]
use ockam::{
    compat::string::{String, ToString},
    println
};

#[cfg(feature = "alloc")]
mod allocator;

#[cfg(feature = "cortexm")]
use panic_itm as _;

#[cfg(feature = "qemu")]
use cortex_m_semihosting::debug;

#[cfg(feature = "atsame54")]
use atsame54_xpro as _;

#[cfg(feature = "stm32f4")]
use stm32f4xx_hal as _;

// - bare metal entrypoint ----------------------------------------------------

#[cfg(feature = "cortexm")]
#[cortex_m_rt::entry]
fn entry() -> ! {

    // - board setup ----------------------------------------------------------

    #[cfg(feature = "bsp_daisy")]
    use daisy_bsp as bsp;
    #[cfg(feature = "bsp_nucleo_h7xx")]
    use nucleo_h7xx as bsp;

    use bsp::{hal, pac};
    use hal::prelude::*;


    let board = bsp::Board::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let _ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                    dp.RCC.constrain(),
                                    &dp.SYSCFG);

    /*let _ccdr = {
        let (pwr, rcc, syscfg) = (dp.PWR.constrain(),
                                  dp.RCC.constrain(),
                                  &dp.SYSCFG);
        let pwrcfg = pwr.vos0(syscfg).freeze();
        let ccdr = rcc.use_hse(hal::time::MegaHertz(16))
            .pll1_strategy(hal::rcc::PllConfigStrategy::Iterative) // pll1 drives system clock
            .sys_ck(480.mhz())                                     // system clock @ 480 MHz
            .pll1_r_ck(480.mhz())                                  // for TRACECK
            .pll3_p_ck(hal::time::Hertz(12_288_000))               // audio clock  @ 12.288 MHz
            .per_ck(4.mhz())                                       // peripheral clock @ 4 MHz
            .freeze(pwrcfg, syscfg);
        unsafe {
            let swo_frequency = 2_000_000;
            let mut cp = cortex_m::Peripherals::steal();
            let dp = pac::Peripherals::steal();
            bsp::itm::enable_itm(&mut cp.DCB,
                                 &dp.DBGMCU,
                                 &mut cp.ITM,
                                 ccdr.clocks.c_ck().0,
                                 swo_frequency);
        }
        ccdr
    };*/

    // - run ockam::node ------------------------------------------------------

    #[cfg(feature = "alloc")]
    allocator::init();

    allocator::stats(1);

    // execute ockam::node main function
    main().unwrap();

    allocator::stats(8);

    println!("Entering main loop");
    loop {}
}

// - ockam::node entrypoint ---------------------------------------------------

use hello_ockam::{Echoer, Hop};
use ockam::{route, Context, Result};

#[ockam::node]
async fn async_main(mut ctx: Context) -> Result<()> {
    allocator::stats(2);

    // Start a worker, of type Echoer, at address "echoer"
    ctx.start_worker("echoer", Echoer).await?;

    allocator::stats(3);

    // Start a worker, of type Hop, at address "h1"
    ctx.start_worker("h1", Hop).await?;

    allocator::stats(4);

    // Send a message to the worker at address "echoer",
    // via the worker at address "h1"
    ctx.send(route!["h1", "echoer"], "Hello Ockam!".to_string()).await?;

    allocator::stats(5);

    // Wait to receive a reply and print it.
    let reply = ctx.receive::<String>().await?;
    println!("App Received: {}", reply); // should print "Hello Ockam!"

    allocator::stats(6);

    // Stop all workers, stop the node, cleanup and return.
    let result = ctx.stop().await;

    allocator::stats(7);

    // exit qemu
    #[cfg(feature = "qemu")]
    {
        debug::exit(debug::EXIT_SUCCESS);
    }

    result
}
