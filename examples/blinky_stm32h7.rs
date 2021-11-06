#![feature(alloc_error_handler)]
#![no_main]
#![no_std]

use panic_itm as _;

#[cfg(feature = "bsp_daisy")]
use daisy_bsp as bsp;
#[cfg(feature = "bsp_nucleo_h7xx")]
use nucleo_h7xx as bsp;

use bsp::hal::prelude::*;
use bsp::led::Led;
use bsp::loggit as println;

mod allocator;


#[cortex_m_rt::entry]
fn main() -> ! {

    // - initialize allocator -------------------------------------------------

    allocator::init();

    // - board setup ----------------------------------------------------------

    let board = bsp::Board::take().unwrap();
    let dp = bsp::pac::Peripherals::take().unwrap();

    let ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                   dp.RCC.constrain(),
                                   &dp.SYSCFG);

    println!("Hello nucleo !");

    let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                 dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                 dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                 dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                 dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                 dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                 dp.GPIOG.split(ccdr.peripheral.GPIOG));

    let mut user_leds = bsp::led::UserLeds::new(pins.user_leds);


    // - main loop ------------------------------------------------------------

    let one_second = ccdr.clocks.sys_ck().0;
    let mut counter = 0;

    loop {
        println!("counter: {}", counter);
        counter += 1;

        user_leds.ld2.on();
        cortex_m::asm::delay(one_second);
        user_leds.ld2.off();
        cortex_m::asm::delay(one_second);
    }
}
