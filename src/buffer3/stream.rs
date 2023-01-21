use super::*;

impl<C: CopyStrategy<T>, P, D, T> Source<T> for Buffer<D, T, C, P>
where
    D: AsRef<[T]>,
{
    fn source(&mut self, into: &mut [T]) -> IO {
        self.copy_into(into)
    }
}

impl<C: CopyStrategy<T>, P, D, T> Sink<T> for Buffer<D, T, C, P>
where
    D: AsMut<[T]>,
{
    fn sink(&mut self, from: &[T]) -> IO {
        self.copy_from(from)
    }
}

impl<'a, S: Source<T>, T> Source<T> for &'a mut S {
    fn source(&mut self, into: &mut [T]) -> IO {
        <S as Source<T>>::source(self, into)
    }
}

impl<'a, S: Sink<T>, T> Sink<T> for &'a mut S {
    fn sink(&mut self, from: &[T]) -> IO {
        <S as Sink<T>>::sink(self, from)
    }
}

pub struct Read<S: io::Read>(pub S);
impl<S: io::Read> Source<u8> for Read<S> {
    fn source(&mut self, into: &mut [u8]) -> IO {
        self.0.read(into)
    }
}

pub struct Write<S: io::Write>(pub S);
impl<S: io::Write> Sink<u8> for Write<S> {
    fn sink(&mut self, from: &[u8]) -> IO {
        self.0.write(from)
    }
}
