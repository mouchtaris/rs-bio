#![doc = include_str!("../doc/crate.md")]

mod buffer3;
pub use buffer3::{
    flow,
    stream,
    Buffer,
    CompactStrategy,
    CopyStrategy,
    SClone,
    SCopy,
    SNone,
    Sink,
    Source,
    IO,
};
