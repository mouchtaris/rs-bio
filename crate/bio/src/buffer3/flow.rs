use super::*;

pub struct EachConsecutiveFlow<D, T, C, P>(pub Buffer<D, T, C, P>);
pub struct EachConsecutive<S, D, T, C, P>(S, Option<Buffer<D, T, C, P>>);

mod each_consecutive;
