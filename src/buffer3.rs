use {
    std::{
        io,
        marker::PhantomData,
        ops::Range,
    },
    tap::Tap,
};
mod buffer;
mod compact_strategy;
mod copy_strategy;
#[cfg(test)]
mod test;

pub mod stream;
pub mod tap;

pub type IO<T = usize> = io::Result<T>;

pub trait Source<T> {
    fn source(&mut self, into: &mut [T]) -> IO;
}

pub trait Sink<T> {
    fn sink(&mut self, from: &[T]) -> IO;
}

pub trait CopyStrategy<T> {
    fn copy_slice(dest: &mut [T], src: &[T]);
}

pub trait CompactStrategy<T> {
    fn compact_within(slice: &mut [T], area: Range<usize>);
}

pub struct SCopy;
pub struct SClone;
pub struct SNone;

pub struct Buffer<D, T, C, P> {
    data: D,
    span: Range<usize>,
    _item_evidence: PhantomData<T>,
    _copy_strategy: PhantomData<C>,
    _compact_strategy: PhantomData<P>,
}

fn transfuse_rec<C, P, D, T>(
    source_done: bool,
    total: usize,
    buffer: &mut Buffer<D, T, C, P>,
    mut source: impl Source<T>,
    mut sink: impl Sink<T>,
) -> IO
where
    C: CopyStrategy<T>,
    P: CompactStrategy<T>,
    D: AsMut<[T]> + AsRef<[T]>,
{
    buffer.compact();

    // Optimize/stabilize: not hitting source after it has returned Ok(0)
    let read = if source_done {
        // We avoid reading source after Ok(0) has been returned, for performance
        // but also to have a deterministic contract for transfuse():
        // We have to assume Ok(0) is final.
        0
    } else {
        // Source will return Ok(0) either because the underlying source
        // is depleted, or the destination sink is "full" (back-pressure),
        // and the buffer has also become full. Thus, every source-read has a
        // zero-length destination slice to be read in, and Ok(0) is returned.
        buffer.read(&mut source)?
    };
    let write = buffer.write(&mut sink)?;

    if read == 0 && write == 0 {
        Ok(total)
    } else {
        transfuse_rec(read == 0, total + write, buffer, source, sink)
    }
}
