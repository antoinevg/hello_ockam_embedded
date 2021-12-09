#![allow(unused_macros)]
#![allow(unused_variables)]

#![no_std]
#![no_main]

//use panic_halt as _;
use panic_semihosting as _;
//use panic_itm as _;

macro_rules! loggit {
    ($($arg:tt)*) => (
        //let itm = unsafe { &mut *cortex_m::peripheral::ITM::ptr() };
        //cortex_m::iprintln!(&mut itm.stim[0], $($arg)*);
        cortex_m_semihosting::hprintln!($($arg)*).unwrap();
    )
}


use cortex_m_rt::entry;

extern crate atsame54_xpro as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::watchdog::{Watchdog, WatchdogTimeout};


#[entry]
fn main() -> ! {
    let cp = CorePeripherals::take().unwrap();
    let mut dp = Peripherals::take().unwrap();

    let mut clocks = GenericClockController::with_internal_32kosc(
        dp.GCLK,
        &mut dp.MCLK,
        &mut dp.OSC32KCTRL,
        &mut dp.OSCCTRL,
        &mut dp.NVMCTRL,
    );

    let mut delay = Delay::new(cp.SYST, &mut clocks);
    delay.delay_ms(400u16);

    let mut pins = hal::Pins::new(dp.PORT);
    let mut led = pins.led.into_open_drain_output(&mut pins.port);

    let mut wdt = Watchdog::new(dp.WDT);
    wdt.start(WatchdogTimeout::Cycles256 as u8);

    let mut counter = 0;

    loop {
        delay.delay_ms(200u8);
        wdt.feed();
        led.set_high().unwrap();
        delay.delay_ms(200u8);
        wdt.feed();
        led.set_low().unwrap();
        loggit!("Counter: {}", counter);
        counter += 1;
    }
}
