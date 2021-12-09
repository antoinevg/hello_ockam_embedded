#![allow(unused_macros)]
#![allow(unused_variables)]

#![no_std]
#![no_main]

use panic_semihosting as _;
use cortex_m_rt::entry;

extern crate atsame54_xpro as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::watchdog::{Watchdog, WatchdogTimeout};


// - println ------------------------------------------------------------------

use core::fmt::{self, Write};

use hal::gpio::v2::{Pins, Pin, PB16, PB17, AlternateC};
use hal::sercom::v2::{IoSet1, Sercom5, uart};
use hal::time::U32Ext;

type Rx = Pin<PB17, AlternateC>;
type Tx = Pin<PB16, AlternateC>;
type Pads = uart::Pads<Sercom5, IoSet1, Rx, Tx>;
type Config = uart::Config<Pads, uart::EightBit>;
type Uart = uart::Uart<Config, uart::Duplex>;

static mut UART: Option<Uart> = None;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        if let Some(uart) = unsafe { UART.as_mut() } {
            let mut buffer = [0u8; 256];
            let mut buffer = BufferWriter::new(&mut buffer[..]);
            writeln!(&mut buffer,  $($arg)*).unwrap();
            for byte in buffer.as_bytes() {
                // NOTE `block!` blocks until `uart.write()` completes and returns
                // `Result<(), Error>`
                nb::block!(uart.write(*byte)).unwrap();
            }
        }
    }};
}

pub struct BufferWriter<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
}

impl<'a> BufferWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        BufferWriter { buffer, cursor: 0 }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[0..self.cursor]
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[0..self.cursor]).unwrap()
    }
}

impl fmt::Write for BufferWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let len = self.buffer.len();
        for (i, &b) in self.buffer[self.cursor..len]
            .iter_mut()
            .zip(s.as_bytes().iter())
        {
            *i = b;
        }
        self.cursor = usize::min(len, self.cursor + s.as_bytes().len());
        Ok(())
    }
}


// - main ---------------------------------------------------------------------

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

    let pins = Pins::new(dp.PORT);
    let mut delay = Delay::new(cp.SYST, &mut clocks);

    // - initialize uart ------------------------------------------------------

    let pads: Pads = uart::Pads::<Sercom5, IoSet1>::default()
        .rx(pins.pb17)
        .tx(pins.pb16);
    let gclk0 = clocks.gclk0();
    let clock = &clocks.sercom5_core(&gclk0).unwrap();
    let config: Config = uart::Config::new(
        &dp.MCLK,
        dp.SERCOM5,
        pads,
        clock.freq()
    );
    let uart: Uart = config
        .baud(115_200.hz(), uart::BaudMode::Arithmetic(uart::Oversampling::Bits8))
        .char_size::<uart::EightBit>()
        .parity(uart::Parity::None)
        .stop_bits(uart::StopBits::OneBit)
        .enable();

    // ------------------------------------------------------------------------

    unsafe { UART.replace(uart); }

    let mut led = pins.pc18.into_mode::<hal::gpio::v2::PushPullOutput>();

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
        println!("Counter: {}", counter);

        counter += 1;
    }
}
