use std::io;

#[doc = include_str!("../doc/Buffer.md")]
pub struct Buffer<D> {
    position: usize,
    limit: usize,
    store: D,
}

/// A Buffer backed by a `Vec<u8>`
pub type OwnedBuffer = Buffer<Vec<u8>>;

impl<D> Buffer<D> {
    /// Create a new empty Buffer.
    pub fn new(store: D) -> Self {
        Self {
            position: 0,
            limit: 0,
            store,
        }
    }

    /// Consume the buffer and return the backing `store`.
    pub fn to_inner(self) -> D {
        self.store
    }

    /// Return the `position` offset.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Return the `limit` offset.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Return the available space for `Self::read()`.
    ///
    /// This is `Self::limit()` - `Self::position()`.
    pub fn available(&self) -> usize {
        self.limit() - self.position()
    }

    /// Return whether the buffer is empty.
    ///
    /// This is `Self::available()`` == 0`.
    pub fn is_empty(&self) -> bool {
        self.available() == 0
    }

    /// Return `store`'s `slice::len()`.
    pub fn len<T>(&self) -> usize
    where
        D: AsRef<[T]>,
    {
        self.store.as_ref().len()
    }

    /// Return the free space for `Self::write()`.
    ///
    /// This is `Self::len()` - `Self::limit()`.
    pub fn free<T>(&self) -> usize
    where
        D: AsRef<[T]>,
    {
        self.len() - self.limit()
    }

    /// Return whether the buffer is full.
    ///
    /// This is `Self::free()`` == 0`.
    pub fn is_full<T>(&self) -> bool
    where
        D: AsRef<[T]>,
    {
        self.free() == 0
    }

    /// Create a new Buffer, which is configured for reading.
    ///
    /// The buffer will have an available space that contains the whole storage
    /// provided.
    ///
    /// This is useful for creating source buffers, which can be used as `io::Read` sources.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> std::io::Result<()> { use bio::Buffer;
    /// let mut src = Buffer::source([0u8, 1, 2]);
    /// let mut dst = Buffer::new([0u8; 3]);
    ///
    /// dst.read(src)?;
    ///
    /// assert_eq!(dst.to_inner(), [0, 1, 2]);
    /// # Ok(()) }
    /// ```
    pub fn source<T>(store: D) -> Self
    where
        D: AsRef<[T]>,
    {
        Self {
            position: 0,
            limit: store.as_ref().len(),
            store,
        }
    }

    /// Compact the buffer.
    ///
    /// Internally, this will move the available and free areas
    /// so that they start from the beginning of the backing storage.
    ///
    /// It is implemented through `slice::copy_within()` and therefore
    /// the time complexity is `O(n)` for the buffer's `Self::len()`.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> std::io::Result<()> { use bio::Buffer;
    /// let mut buf = Buffer::new([0u8; 5]);
    ///
    /// // Write up to 4
    /// buf.read([1u8, 2, 3, 4].as_ref())?;
    ///
    /// // Read up to 2
    /// buf.write([0u8; 2].as_mut())?;
    ///
    /// // Now position is at 2 and limit at 4
    /// assert_eq!(buf.position(), 2);
    /// assert_eq!(buf.limit(), 4);
    ///
    /// // After compacting, these offsets are translated from 0
    /// buf.compact();
    /// assert_eq!(buf.position(), 0);
    /// assert_eq!(buf.limit(), 2);
    /// # Ok(()) }
    pub fn compact<T>(&mut self)
    where
        D: AsMut<[T]>,
        T: Copy,
    {
        let Self {
            position,
            limit,
            store,
        } = self;

        let store = store.as_mut();

        let available = *position..*limit;
        store.copy_within(available, 0);

        *limit -= *position;
        *position = 0;
    }
}

impl<D: AsRef<[u8]>> Buffer<D> {
    /// Write bytes to a sink into from the buffer's available area.
    ///
    /// This will advance the `Self::position()` pointer, reducing this way
    /// the available area.
    ///
    /// Returns the number of bytes written.
    pub fn write(&mut self, mut sink: impl io::Write) -> io::Result<usize> {
        sink.write(self.as_read()).map(|n| {
            self.position += n;
            n
        })
    }

    /// Return the available area of the buffer's backing store as a slice.
    pub fn as_read(&self) -> &[u8] {
        let Self {
            position,
            limit,
            store,
            ..
        } = self;
        &store.as_ref()[*position..*limit]
    }
}

impl<D: AsMut<[u8]>> Buffer<D> {
    /// Read bytes from a source into the buffer's free area.
    ///
    /// This will advance the `Self::limit()` pointer, reducing this way
    /// the free area.
    ///
    /// Returns the number of bytes read.
    pub fn read(&mut self, mut source: impl io::Read) -> io::Result<usize> {
        source.read(self.as_write()).map(|n| {
            self.limit += n;
            n
        })
    }

    /// Return the free area of the buffers backing store as a slice.
    pub fn as_write(&mut self) -> &mut [u8] {
        let Self { limit, store, .. } = self;
        &mut store.as_mut()[*limit..]
    }

    /// Completely copy a source to a sink.
    ///
    /// This function will repeatedly read from the source and write to
    /// the sink, `Self::compact()`ing in-between.
    ///
    /// # Method of transfusion
    ///
    /// Transfusion happens in cycles. On each cycle the buffer is
    /// - `Self::compact()`ed
    /// - `Self::read()`ed into
    /// - `Self::write()`en from
    ///
    /// This cycle goes on until both `read` and `write` give back a zero number of bytes
    /// moved. This implies that a stale-mate situation has arisen, either by fact of source
    /// reaching end-of-stream, or that both sink and buffer are "full" and nothing
    /// more can happen (back-pressure).
    ///
    /// # WARNING
    ///
    /// In case of a stale-mate (sink full), the contents of the input stream that are still in
    /// the buffer will be lost after the buffer is dropped!
    ///
    /// However, this scenario should only arise with finite capacity sinks (such as static length
    /// arrays).
    ///
    /// # Return value
    ///
    /// Returns the total number of bytes transfused.
    pub fn transfuse(&mut self, source: impl io::Read, sink: impl io::Write) -> io::Result<usize>
    where
        D: AsRef<[u8]>,
    {
        transfuse(0, self, source, sink)
    }
}

/// Return the backing storage as a slice.
impl<D, T> AsRef<[T]> for Buffer<D>
where
    D: AsRef<[T]>,
{
    fn as_ref(&self) -> &[T] {
        &self.store.as_ref()
    }
}

/// Return the backing storage as a mutable slice.
impl<D, T> AsMut<[T]> for Buffer<D>
where
    D: AsMut<[T]>,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.store.as_mut()
    }
}

/// A Buffer can be also seen `io::Read`, reading from and reducing its available area.
impl<D> io::Read for Buffer<D>
where
    D: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.write(buf)
    }
}

/// A Buffer can be also seen `io::Write`, writing to and reducing its free area.
impl<D> io::Write for Buffer<D>
where
    D: AsMut<[u8]>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.read(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn transfuse<D>(
    total: usize,
    buffer: &mut Buffer<D>,
    mut source: impl io::Read,
    mut sink: impl io::Write,
) -> io::Result<usize>
where
    D: AsRef<[u8]> + AsMut<[u8]>,
{
    buffer.compact();

    // These are cheap enough to use directly.
    // Checking separately for is_empty()/is_full() would be an overhead.
    //
    // TODO: optimise transfuse
    // What can be improved is not hitting the source at all, once it has
    // returned 0.
    let read = buffer.read(&mut source)?;
    let written = buffer.write(&mut sink)?;

    match (read, written) {
        (0, 0) => Ok(total),

        // Count bytes on the out-stream.
        //
        // This SHOULD BE a tail-recursion.
        (_, n) => transfuse(total + n, buffer, source, sink),
    }
}
