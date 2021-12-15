use tracing::{Event, Id, Metadata, Subscriber, dispatcher::Dispatch, field::{self, Field}, span::{Attributes, Record}};


// - setup --------------------------------------------------------------------

pub fn setup_tracing() {
    let subscriber = EmbeddedSubscriber::new();
    let dispatch = Dispatch::new(subscriber);
    tracing::dispatcher::set_global_default(dispatch)
        .expect("global default dispatcher for tracing is already set");
}


// - EmbeddedSubscriber -------------------------------------------------------

struct EmbeddedSubscriber;

impl EmbeddedSubscriber {
    fn new() -> Self {
        Self
    }
}

// https://docs.rs/tracing-subscriber/0.3.3/tracing_subscriber/fmt/struct.Subscriber.html
// tracing.git/tracing-subscriber/src/fmt/format/pretty.rs

// TODO add a SubscriberBuilder so we can set log levels etc.

impl Subscriber for EmbeddedSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        //tprintln!("EmbeddedSubscriber::enabled");
        true
    }

    fn new_span(&self, _span: &Attributes<'_>) -> Id {
        tprintln!("EmbeddedSubscriber::new_span");
        tracing::span::Id::from_u64(0xAAAA)
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {
        tprintln!("EmbeddedSubscriber::record");
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {
        tprintln!("EmbeddedSubscriber::record_follows_from");
    }

    fn event(&self, event: &Event<'_>) {
        let mut visitor = EmbeddedVisitor::new();
        //tprintln!("event: {:?}", event);
        event.record(&mut visitor);
    }

    fn enter(&self, _span: &Id) {
        tprintln!("EmbeddedSubscriber::enter");
    }

    fn exit(&self, _span: &Id) {
        tprintln!("EmbeddedSubscriber::exit");
    }
}


// - EmbeddedVisitor ----------------------------------------------------------

struct EmbeddedVisitor;

impl EmbeddedVisitor {
    fn new() -> Self {
        Self
    }
}

impl field::Visit for EmbeddedVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            tprintln!("{}", value);
        } else {
            //self.record_debug(field, &value)
            tprintln!("{}: {}", field.name(), value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn core::fmt::Debug) {
        if field.name() == "message" {
            tprintln!("d: {:?}", value);
        } else {
            //self.record_debug(field, &value)
            tprintln!("d:{}: {:?}", field.name(), value);
        }
    }
}


// - deleteme -----------------------------------------------------------------

#[macro_export]
macro_rules! tprintln {
    ($($arg:tt)*) => {{
        #[cfg(feature="log-itm")]
        {
            // give the itm buffer time to empty
            cortex_m::asm::delay(96_000_000 / 32);
            // print output using itm peripheral
            let itm = unsafe { &mut *cortex_m::peripheral::ITM::ptr() };
            cortex_m::iprintln!(&mut itm.stim[0], $($arg)*);
        }

        #[cfg(feature="log-semihosting")]
        cortex_m_semihosting::hprintln!($($arg)*).unwrap();

        // dummy fallback definition
        #[cfg(not(any(feature="log-semihosting", feature="log-itm")))]
        {
            use ockam_core::compat::io::Write;
            let mut buffer = [0 as u8; 1];
            let mut cursor = ockam_core::compat::io::Cursor::new(&mut buffer[..]);

            match write!(&mut cursor, $($arg)*) {
                Ok(()) => (),
                Err(_) => (),
            }
        }
    }};
}

use crate::tprintln;
