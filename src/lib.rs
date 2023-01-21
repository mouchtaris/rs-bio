#![doc = include_str!("../doc/crate.md")]

//mod buffer;
//pub mod stream;
//#[cfg(test)]
//mod example;

//pub use buffer::{
//    Buffer,
//    OwnedBuffer,
//};

//pub mod buffer2;
//mod example2;
mod buffer3;
pub use buffer3::{
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
