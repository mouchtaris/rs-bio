use std::{
    cmp,
    io,
    marker::{
        self,
        PhantomData as __,
    },
    mem,
    ops::Range,
};

pub trait Source<T> {
    fn source(&mut self, into: &mut [T]) -> io::Result<usize>;
}

pub trait Sink<T> {
    fn sink(&mut self, from: &[T]) -> io::Result<usize>;
    fn flush(&mut self) -> io::Result<()>;
}

pub trait IntoSource<T> {
    type Source: Source<T>;
    fn into_source(self) -> Self::Source;
}

pub trait Buffer<T>: AsRef<[T]> + AsMut<[T]> {
    fn span_mut(&mut self) -> (&mut usize, &mut usize);
    fn span(&self) -> Range<usize>;
    fn compact(&mut self);

    fn len(&self) -> usize {
        self.as_ref().len()
    }
    fn position(&self) -> usize {
        self.span().start
    }
    fn limit(&self) -> usize {
        self.span().end
    }
    fn available(&self) -> usize {
        self.span().len()
    }
    fn free(&self) -> usize {
        self.len() - self.limit()
    }
    fn is_empty(&self) -> bool {
        self.available() == 0
    }
    fn is_full(&self) -> bool {
        self.free() == 0
    }

    fn as_read(&self) -> &[T] {
        &self.as_ref()[self.span()]
    }

    fn as_write(&mut self) -> &mut [T] {
        let area = self.limit()..;
        &mut self.as_mut()[area]
    }

    fn read(&mut self, mut from: impl Source<T>) -> io::Result<usize> {
        from.source(self.as_write()).map(|n| {
            let (_, limit) = self.span_mut();
            *limit += n;
            n
        })
    }

    fn write(&mut self, mut to: impl Sink<T>) -> io::Result<usize> {
        to.sink(self.as_read()).map(|n| {
            let (position, _) = self.span_mut();
            *position += n;
            n
        })
    }

    fn transfuse(&mut self, source: impl Source<T>, sink: impl Sink<T>) -> io::Result<usize> {
        fn transfuse<T>(
            total: usize,
            mut source_done: bool,
            mut buffer: impl Buffer<T>,
            mut source: impl Source<T>,
            mut sink: impl Sink<T>,
        ) -> io::Result<usize> {
            buffer.compact();

            // read()/write() are cheap enough to use directly.
            // Checking separately for is_empty()/is_full() would be an overhead.
            let read = if source_done {
                0
            } else {
                buffer.read(&mut source)?.tap(|&n| source_done = n == 0)
            };
            let written = buffer.write(&mut sink)?;

            match (read, written) {
                (0, 0) => Ok(total),

                // Count bytes on the out-stream.
                //
                // This SHOULD BE a tail-recursion.
                (_, n) => transfuse(total + n, source_done, buffer, source, sink),
            }
        }
        transfuse(0, false, self, source, sink)
    }
}

impl<'a, S: ?Sized, T> Source<T> for &'a mut S
where
    S: Source<T>,
{
    fn source(&mut self, into: &mut [T]) -> io::Result<usize> {
        <S as Source<T>>::source(self, into)
    }
}

impl<'a, T> Source<T> for &'a [T] {
    fn source(&mut self, into: &mut [T]) -> io::Result<usize> {
        todo!()
    }
}

impl<T> Source<T> for [T] {
    fn source(&mut self, into: &mut [T]) -> io::Result<usize> {
        todo!()
    }
}

impl<T, const N: usize> Source<T> for [T; N] {
    fn source(&mut self, into: &mut [T]) -> io::Result<usize> {
        todo!()
    }
}

impl<'a, S: ?Sized, T> Sink<T> for &'a mut S
where
    S: Sink<T>,
{
    fn sink(&mut self, from: &[T]) -> io::Result<usize> {
        <S as Sink<T>>::sink(self, from)
    }

    fn flush(&mut self) -> io::Result<()> {
        <S as Sink<T>>::flush(self)
    }
}

impl<T> Sink<T> for [T] {
    fn sink(&mut self, from: &[T]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl<T, const N: usize> Sink<T> for [T; N] {
    fn sink(&mut self, from: &[T]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl<'a, S, T> Buffer<T> for &'a mut S
where
    S: ?Sized + Buffer<T>,
{
    fn span_mut(&mut self) -> (&mut usize, &mut usize) {
        <S as Buffer<T>>::span_mut(self)
    }

    fn span(&self) -> Range<usize> {
        <S as Buffer<T>>::span(self)
    }

    fn compact(&mut self) {
        <S as Buffer<T>>::compact(self)
    }
}

pub struct Read<R>(pub R);
impl<R: io::Read> Source<u8> for Read<R> {
    fn source(&mut self, into: &mut [u8]) -> io::Result<usize> {
        self.0.read(into)
    }
}

pub struct Write<W>(pub W);
impl<W: io::Write> Sink<u8> for Write<W> {
    fn sink(&mut self, from: &[u8]) -> io::Result<usize> {
        self.0.write(from)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub struct Data<B>(pub B);
impl<B: Buffer<T>, T> Source<T> for Data<B> {
    fn source(&mut self, into: &mut [T]) -> io::Result<usize> {
        self.0.write(into)
    }
}
impl<B: Buffer<T>, T> Sink<T> for Data<B> {
    fn sink(&mut self, from: &[T]) -> io::Result<usize> {
        self.0.read(from)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub type Copy<D> = SpannedBuffer<(), D>;
pub type Move<D> = SpannedBuffer<((),), D>;
pub struct SpannedBuffer<C, D> {
    span: Range<usize>,
    store: D,
    __: __<C>,
}
impl<C, D> SpannedBuffer<C, D> {
    pub fn new(store: D) -> Self {
        Self {
            __,
            span: 0..0,
            store,
        }
    }

    pub fn as_source<T>(mut self) -> Self
    where
        D: AsRef<[T]>,
    {
        let Self {
            span: Range { end, .. },
            store,
            ..
        } = &mut self;
        *end = store.as_ref().len();
        self
    }
}
impl<T, C, D: AsRef<[T]>> AsRef<[T]> for SpannedBuffer<C, D> {
    fn as_ref(&self) -> &[T] {
        self.store.as_ref()
    }
}
impl<T, C, D: AsMut<[T]>> AsMut<[T]> for SpannedBuffer<C, D> {
    fn as_mut(&mut self) -> &mut [T] {
        self.store.as_mut()
    }
}
impl<T, C: Compactor<T>, D: AsMut<[T]> + AsRef<[T]>> Buffer<T> for SpannedBuffer<C, D> {
    fn span_mut(&mut self) -> (&mut usize, &mut usize) {
        let Self {
            span: Range { start, end },
            ..
        } = self;
        (start, end)
    }

    fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn compact(&mut self) {
        let area = self.span();
        C::compact(self.as_mut(), area, 0);
    }
}

pub trait Compactor<T> {
    fn compact(slice: &mut [T], area: Range<usize>, dest: usize);
}
impl<T: marker::Copy> Compactor<T> for () {
    fn compact(slice: &mut [T], area: Range<usize>, dest: usize) {
        slice.copy_within(area, dest)
    }
}
impl<T> Compactor<T> for ((),) {
    fn compact(slice: &mut [T], area: Range<usize>, dest_off: usize) {
        let (dest, src) = slice.split_at_mut(area.start);

        let dest = &mut dest[dest_off..];

        for i in 0..area.len() {
            mem::swap(&mut dest[i], &mut src[i]);
        }
    }
}

pub trait Tap {
    fn tap(self, block: impl FnOnce(&Self)) -> Self
    where
        Self: Sized,
    {
        block(&self);
        self
    }
}
impl<T> Tap for T {}
