#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_std]
#![no_main]

use panic_itm as _;
use cortex_m_rt::entry;

use stm32f4xx_hal as hal;
use hal::prelude::*;
use hal::delay::Delay;
use hal::timer::Timer;
use hal::hal as embedded_hal;
use embedded_hal::blocking;
use hal::pac;


// - println ------------------------------------------------------------------

macro_rules! println {
    ($($arg:tt)*) => {{
        let itm = unsafe { &mut *cortex_m::peripheral::ITM::ptr() };
        cortex_m::iprintln!(&mut itm.stim[0], $($arg)*);
    }};
}


// - entry --------------------------------------------------------------------

#[entry]
fn entry() -> ! {
    match main() {
        Ok(_) => (),
        Err(e) => {
            println!("main error: {:?}", e);
        }
    }

    loop {
        cortex_m::asm::wfi();
    }
}


fn main() -> Result<(), u32> {

    // - configure board ------------------------------------------------------

    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    //let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

    let clocks = rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .pclk1(24.mhz())
        .freeze(); //&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);


    // - initialize user leds -------------------------------------------------

    let gpiod = dp.GPIOD.split();
    let mut led = gpiod.pd12.into_push_pull_output();

    delay.delay_ms(200u8);

    // - main loop ------------------------------------------------------------

    println!("\nentering main loop");

    let mut counter = 0;

    loop {
        println!("counter: {}", counter);

        led.toggle().ok();
        delay.delay_ms(200u8);
        if counter >= 1_000_000 {
            break;
        } else {
            counter += 1;
        }
    }

    Ok(())
}
