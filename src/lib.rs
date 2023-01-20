#![doc = include_str!("../doc/crate.md")]

mod buffer;
pub mod stream;

pub use {
    buffer::{Buffer, OwnedBuffer},
    stream::Source,
};
