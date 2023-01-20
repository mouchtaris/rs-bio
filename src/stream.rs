use std::io;

pub mod chunk;

mod impl_streams_for_buffer;
mod impl_streams_for_slice;

pub trait Source<T> {
    fn read(&mut self, into_sink: &mut [T]) -> io::Result<usize>;
}

pub trait Sink<T> {
    fn write(&mut self, from_source: &[T]) -> io::Result<usize>;
}

pub trait Flow<T> {
    fn transfuse(&mut self, source: impl Source<T>, sink: impl Sink<T>) -> io::Result<usize>;
}
