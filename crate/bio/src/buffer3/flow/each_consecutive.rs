use super::*;

impl<S, D, T, C, P> EachConsecutive<S, D, T, C, P> {
    pub fn new(source: S, buf: Buffer<D, T, C, P>) -> Self {
        Self(source, Some(buf))
    }
}

impl<D, T, C, P> Flow<T, Buffer<D, T, C, P>> for EachConsecutiveFlow<D, T, C, P>
where
    Buffer<D, T, C, P>: Clone,
    D: AsRef<[T]> + AsMut<[T]>,
{
    type Source<S: Source<T>> = EachConsecutive<S, D, T, C, P>;

    fn flow<S: Source<T>>(&self, inp: S) -> Self::Source<S> {
        EachConsecutive::new(inp, self.0.clone())
    }
}

impl<S, D, T, C, P> Source<Buffer<D, T, C, P>> for EachConsecutive<S, D, T, C, P>
where
    Buffer<D, T, C, P>: Clone,
    D: AsRef<[T]> + AsMut<[T]>,
    S: Source<T>,
{
    fn source(&mut self, into: &mut [Buffer<D, T, C, P>]) -> IO {
        let Self(source, buf_opt_ref) = self;

        // Target will keep track of how many items have been moved.
        // We return it at the end of the function.
        let mut target = 0;

        // Take the buffer from the option, so we own it and
        // can possibly place it in destination.
        let buf_opt = buf_opt_ref.take();

        // If we still have a buffer...
        // (We will not have a buffer after source has reported read()==0 for
        // the first time. So this guard is for subsequent calls of this function).
        if let Some(mut buf) = buf_opt {
            loop {
                match (buf.is_full(), into.get_mut(target)) {
                    // Destination slice is full.
                    (_, None) => {
                        // Destination might not be full in future invocations of
                        // this function, so put the buffer back.
                        *buf_opt_ref = Some(buf);
                        break;
                    }

                    // Buffer is full and destination is available.
                    (true, Some(cell)) => {
                        // Place an item.
                        *cell = buf.clone();
                        // Clean the buffer.
                        buf.clear();
                        // Increase destination.
                        target += 1;

                        // Continue the loop.
                        // !break
                    }

                    // Buffer is not full and destination is available
                    (false, Some(cell)) => {
                        // Try to fill the buffer

                        // Source is done
                        if buf.read(&mut *source)? == 0 {
                            // And buffer empty
                            if buf.is_empty() {
                                // Done without placing the buffer back.
                                // Future invocations will directly return Ok(0).
                                break;
                            } else {
                                // Buffer is not empty (and also not full).

                                // With a non-empty and not-full buffer, we
                                // have unused data in the pipes.

                                // We place the buffer and as a final item in destination.
                                *cell = buf;
                                // Increase item counter
                                target += 1;
                                // Done
                                break;
                            }
                        } else {
                            // We read some new data into the buffer.
                            // Let the loop resume.

                            // !break
                        }
                    }
                }
            }
        }
        Ok(target)
    }
}
