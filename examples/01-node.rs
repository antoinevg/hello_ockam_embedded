#![cfg_attr(all(feature = "alloc", feature = "cortexm"), feature(alloc_error_handler))]
#![cfg_attr(all(not(feature = "std"), feature = "cortexm"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "cortexm"), no_main)]


// - bare metal entrypoint ----------------------------------------------------

#[cfg(feature = "cortexm")]
use ockam::println;

#[cfg(all(feature = "alloc", feature = "cortexm"))]
mod allocator;

#[cfg(feature = "cortexm")]
use panic_itm as _;

#[cfg(feature = "qemu")]
use cortex_m_semihosting::debug;

#[cfg(feature = "atsame54")]
use atsame54_xpro as _;

#[cfg(feature = "stm32f4")]
use stm32f4xx_hal as _;

#[cfg(feature = "stm32h7")]
use stm32h7xx_hal as _;

// - cortex-m exception handlers ----------------------------------------------

#[cfg(feature = "cortexm")]
#[cortex_m_rt::exception]
fn DefaultHandler(irqn: i16) {
    println!("DefaultHandler: {}", irqn);

    loop {}
}

#[cfg(feature = "cortexm")]
#[cortex_m_rt::exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    println!("HardFault: {:#?}", ef);

    loop {}
}

// - bare metal entrypoint ----------------------------------------------------

#[cfg(feature = "cortexm")]
#[cortex_m_rt::entry]
fn entry() -> ! {

    // - board setup ----------------------------------------------------------

    use daisy_bsp as daisy;
    use daisy::hal::prelude::*;


    let board = daisy::Board::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();

    let _ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                    dp.RCC.constrain(),
                                    &dp.SYSCFG);

    // - run ockam::node ------------------------------------------------------

    #[cfg(feature = "alloc")]
    allocator::init();

    // execute ockam::node main function
    main().unwrap();

    loop { }
}


// - ockam::node entrypoint ---------------------------------------------------

use ockam::{Context, Result};

#[ockam::node]
async fn async_main(mut ctx: Context) -> Result<()> {

    // Stop the node as soon as it starts.
    println!("Stop the node as soon as it starts.");
    let result = ctx.stop().await;

    // exit qemu
    #[cfg(feature = "qemu")]
    {
        debug::exit(debug::EXIT_SUCCESS);
    }

    result
}
