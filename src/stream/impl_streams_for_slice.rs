use {
    super::{
        Sink,
        Source,
    },
    std::{
        cmp::min,
        mem,
    },
};

fn sink_into_slice<T: Copy>(source: &[T], sink: &mut [T]) -> usize {
    let n = min(source.len(), sink.len());

    sink[0..n].copy_from_slice(&source[0..n]);
    n
}

impl<'a, T: Copy> Source<T> for &'a [T] {
    fn read(&mut self, into_sink: &mut [T]) -> std::io::Result<usize> {
        let n = sink_into_slice(&self, into_sink);
        *self = &self[n..];
        Ok(n)
    }
}

impl<'a, T: Copy> Sink<T> for &'a mut [T] {
    fn write(&mut self, from_source: &[T]) -> std::io::Result<usize> {
        let n = sink_into_slice(from_source, self);
        let (_, rest) = mem::replace(self, &mut []).split_at_mut(n);
        *self = rest;
        Ok(n)
    }
}
