use super::{Sink, Source};
use crate::Buffer;

impl<T, D> Source<T> for Buffer<D>
where
    D: AsRef<[T]>,
    T: Copy,
{
    fn read(&mut self, mut into_sink: &mut [T]) -> std::io::Result<usize> {
        self.write(&mut into_sink)
    }
}

impl<T, D> Sink<T> for Buffer<D>
where
    D: AsMut<[T]>,
    T: Copy,
{
    fn write(&mut self, mut from_source: &[T]) -> std::io::Result<usize> {
        self.read(&mut from_source)
    }
}
