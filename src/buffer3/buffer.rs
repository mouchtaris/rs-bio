use super::*;

impl<D, T, C, P> Buffer<D, T, C, P> {
    fn new(data: D) -> Self {
        Self {
            data,
            span: 0..0,
            _item_evidence: PhantomData,
            _copy_strategy: PhantomData,
            _compact_strategy: PhantomData,
        }
    }
}

impl<D, T> Buffer<D, T, SNone, SNone> {
    pub fn from(data: D) -> Self {
        Self::new(data)
    }
}

impl<D, T: Clone> Buffer<D, T, SClone, SNone> {
    pub fn from_clone(data: D) -> Self {
        Self::new(data)
    }
}

impl<D, T: Copy> Buffer<D, T, SCopy, SCopy> {
    pub fn from_copy(data: D) -> Self {
        Self::new(data)
    }
}

impl<C, P, D, T> Buffer<D, T, C, P>
where
    D: AsRef<[T]>,
{
    pub fn as_source(mut self) -> Self {
        self.span = 0..self.data.as_ref().len();
        self
    }
    pub fn as_read(&self) -> &[T] {
        let Self { data, span, .. } = self;
        &data.as_ref()[span.clone()]
    }
    pub fn write(&mut self, mut into: impl Sink<T>) -> IO {
        into.sink(self.as_read()).tap_ok(|n| self.span.start += n)
    }
}

impl<C, P, D, T> Buffer<D, T, C, P>
where
    D: AsMut<[T]>,
{
    pub fn as_write(&mut self) -> &mut [T] {
        let Self {
            data,
            span: Range { end, .. },
            ..
        } = self;
        &mut data.as_mut()[*end..]
    }
    pub fn read(&mut self, mut from: impl Source<T>) -> IO {
        from.source(self.as_write()).tap_ok(|n| self.span.end += n)
    }
}

impl<C: CopyStrategy<T>, P, D, T> Buffer<D, T, C, P> {
    fn copy_slice(dest: &mut [T], src: &[T]) -> usize {
        let n = std::cmp::min(dest.len(), src.len());
        let src = &src[..n];
        let dest = &mut dest[..n];
        C::copy_slice(dest, src);
        n
    }
}

impl<C: CopyStrategy<T>, P, D, T> Buffer<D, T, C, P>
where
    D: AsRef<[T]>,
{
    pub fn copy_into(&mut self, into: &mut [T]) -> IO {
        let n = Self::copy_slice(into, self.as_read());
        self.span.start += n;
        Ok(n)
    }
}

impl<C: CopyStrategy<T>, P, D, T> Buffer<D, T, C, P>
where
    D: AsMut<[T]>,
{
    pub fn copy_from(&mut self, from: &[T]) -> IO {
        let n = Self::copy_slice(self.as_write(), from);
        self.span.end += n;
        Ok(n)
    }
}

impl<C, P: CompactStrategy<T>, D, T> Buffer<D, T, C, P>
where
    D: AsMut<[T]>,
{
    pub fn compact(&mut self) {
        let Self {
            span: Range { start, end },
            data,
            ..
        } = self;
        P::compact_within(data.as_mut(), *start..*end);
        *end -= *start;
        *start = 0;
    }
}

impl<C: CopyStrategy<T>, P: CompactStrategy<T>, D, T> Buffer<D, T, C, P>
where
    D: AsMut<[T]> + AsRef<[T]>,
{
    pub fn transfuse(&mut self, source: impl Source<T>, sink: impl Sink<T>) -> IO {
        transfuse_rec(false, 0, self, source, sink)
    }
}
