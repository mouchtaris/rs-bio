use super::Source;
use crate::Buffer;

impl<S: Source<T>, T> Chunk<T> for S {}

pub trait Chunk<T>: Source<T> {
    fn chunk<const N: usize>(&mut self) -> Chunked<Self, T, N, DefaultCloner<T, N>>
    where
        Buffer<[T; N]>: Default + Clone,
    {
        self.chunk_with_buffer(<_>::default())
    }

    fn chunk_with_buffer<const N: usize>(
        &mut self,
        buffer: Buffer<[T; N]>,
    ) -> Chunked<Self, T, N, DefaultCloner<T, N>>
    where
        Buffer<[T; N]>: Clone,
    {
        self.chunk_with_buffer_and_cloner(buffer, <_>::clone)
    }

    fn chunk_with_buffer_and_cloner<const N: usize, C>(
        &mut self,
        buffer: Buffer<[T; N]>,
        cloner: C,
    ) -> Chunked<Self, T, N, C>
    where
        C: Cloner<T, N>,
    {
        Chunked {
            source: self,
            buffer,
            cloner,
        }
    }
}

pub trait Cloner<T, const N: usize>: FnMut(&Buffer<[T; N]>) -> Buffer<[T; N]> {}
impl<S: FnMut(&Buffer<[T; N]>) -> Buffer<[T; N]>, T, const N: usize> Cloner<T, N> for S {}

pub type DefaultCloner<T, const N: usize> = fn(&Buffer<[T; N]>) -> Buffer<[T; N]>;

pub struct Chunked<'s, S: ?Sized, T, const N: usize, C> {
    source: &'s mut S,
    buffer: Buffer<[T; N]>,
    cloner: C,
}

impl<'s, S, T, const N: usize, C> Source<Buffer<[T; N]>> for Chunked<'s, S, T, N, C>
where
    S: Source<T>,
    C: Cloner<T, N>,
{
    fn read(&mut self, into_sink: &mut [Buffer<[T; N]>]) -> std::io::Result<usize> {
        let Self {
            source,
            buffer,
            cloner,
            ..
        } = self;

        let mut sink_idx = 0;
        loop {
            match into_sink.get_mut(sink_idx) {
                None => break Ok(sink_idx),
                Some(cell) => loop {
                    let read = buffer.read(*source)?;
                    if buffer.is_full() {
                        *cell = cloner(&buffer);
                        buffer.clear();
                        sink_idx += 1;
                        break;
                    }
                    if read == 0 {
                        if !buffer.is_empty() {
                            return broken_pipe_error::<N, _>();
                        }
                        return Ok(sink_idx);
                    }
                },
            }
        }
    }
}

fn broken_pipe_error<const N: usize, T>() -> std::io::Result<T> {
    Err(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        format!("Source ended before we can buffer {N} items"),
    ))
}

#[test]
fn chunk_test() -> std::io::Result<()> {
    let mut source = Buffer::source([1u8, 2, 3, 4, 5, 6, 7, 8]);
    let mut dest = Buffer::new([<_>::default()]);

    let mut source = source.chunk::<3>();

    dest.read(&mut source)?;
    assert_eq!(dest.as_read().first().unwrap().as_read(), [1, 2, 3]);

    dest.clear();
    dest.read(&mut source)?;
    assert_eq!(dest.as_ref().first().unwrap().as_read(), [4, 5, 6]);

    dest.clear();
    let pipe_error = dest.read(&mut source);
    assert_eq!(
        pipe_error.err().unwrap().kind(),
        std::io::ErrorKind::BrokenPipe
    );

    Ok(())
}

#[test]
fn chunk_multi_read_test() -> std::io::Result<()> {
    let mut source = Buffer::source([1u8, 2, 3, 4, 5, 6, 7, 8]);
    let mut dest = Buffer::new([<_>::default(), <_>::default()]);

    let mut source = source.chunk::<3>();

    dest.read(&mut source)?;
    let items = &dest.as_read();
    let items = [items[0].as_read(), items[1].as_read()];
    assert_eq!(items, [[1, 2, 3], [4, 5, 6]]);

    dest.clear();
    let actual = dest.read(&mut source).err().unwrap().kind();
    assert_eq!(actual, std::io::ErrorKind::BrokenPipe);

    Ok(())
}

#[test]
fn chunk_multi_read_with_graceful_end_test() -> std::io::Result<()> {
    let mut source = Buffer::source([1u8, 2, 3, 4, 5, 6, 7, 8, 9]);
    let mut dest = Buffer::new([<_>::default(), <_>::default(), <_>::default()]);

    let mut source = source.chunk::<3>();

    dest.read(&mut source)?;
    let items = &dest.as_read();
    let items = [items[0].as_read(), items[1].as_read(), items[2].as_read()];
    assert_eq!(items, [[1, 2, 3], [4, 5, 6], [7, 8, 9],]);

    dest.clear();
    let actual = dest.read(&mut source).ok().unwrap();
    assert_eq!(actual, 0);

    Ok(())
}
