#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate tracing;

pub mod allocator;
pub mod tracing_subscriber;
