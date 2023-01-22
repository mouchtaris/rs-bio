use super::*;

pub struct EachConsecutive<S, D, T, C, P>(pub S, pub Option<Buffer<D, T, C, P>>);

mod each_consecutive;
