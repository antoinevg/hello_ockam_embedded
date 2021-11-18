use core::alloc::Layout;

#[cfg(not(feature = "debug_alloc"))]
use alloc_cortex_m::CortexMHeap;

// - heap ---------------------------------------------------------------------

#[cfg(feature = "qemu")]
const HEAP_SIZE: usize = 1024 * 768; // in bytes
#[cfg(feature = "atsame54")]
const HEAP_SIZE: usize = 1024 *  64; // in bytes
#[cfg(feature = "stm32f4")]
const HEAP_SIZE: usize = 1024 * 128; // in bytes
#[cfg(feature = "stm32h7")]
const HEAP_SIZE: usize = 1024 * 256; // in bytes

#[cfg(not(feature = "debug_alloc"))]
#[global_allocator]
#[cfg_attr(feature = "stm32h7", link_section = ".axisram_heap")]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[cfg(feature = "debug_alloc")]
use ockam_executor::debug_alloc::ALLOCATOR;

// - initialization -----------------------------------------------------------

pub fn init() {
    #[cfg(not(feature = "debug_alloc"))]
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE)
    }
    #[cfg(feature = "debug_alloc")]
    ockam_executor::debug_alloc::init(HEAP_SIZE);
}

// - stats --------------------------------------------------------------------


pub fn stats(id: usize) {
    debug!("#{} => Heap usage: {} / {}  free: {}\n",
           id,
           ALLOCATOR.used(),
           HEAP_SIZE,
           ALLOCATOR.free());
}



// - error handler ------------------------------------------------------------

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    error!("examples/allocator.rs - alloc error: {:?}", layout);
    cortex_m::asm::bkpt();
    loop {}
}
