//#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "cortexm"), no_std)]


mod echoer;
pub use echoer::*;

mod hop;
pub use hop::*;


// - board helpers ------------------------------------------------------------
// TODO move these upstream

#[cfg(feature = "bsp_nucleo_h7xx")]
pub mod boards {
    use nucleo_h7xx as bsp;
    use bsp::{hal, pac};

    pub fn freeze_clocks_with_config(
        pwr: hal::pwr::Pwr,
        rcc: hal::rcc::Rcc,
        syscfg: &hal::device::SYSCFG,
        configure: fn(pwrcfg: hal::pwr::PowerConfiguration,
                      rcc: hal::rcc::Rcc,
                      syscfg: &hal::device::SYSCFG) -> hal::rcc::Ccdr
    ) -> hal::rcc::Ccdr {

        let mut cp = unsafe { cortex_m::Peripherals::steal() };
        let dp = unsafe { pac::Peripherals::steal() };

        // link SRAM3 power state to CPU1
        dp.RCC.ahb2enr.modify(|_, w| w.sram3en().set_bit());

        let pwrcfg = pwr.smps().vos0(syscfg).freeze();
        let ccdr = configure(pwrcfg, rcc, syscfg);

        unsafe {
            let swo_frequency = 4_000_000;
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

    }
}
