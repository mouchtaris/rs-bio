fn main() {}

#[cfg(test)]
use bio::*;

#[test]
fn example() -> IO<()> {
    // In this example we will convert a byte stream into a stream of u32.

    // We define the input stream. It returns 16 bytes, from 0x0 to 0xf.
    let byte_source =
        stream::Read([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15u8].as_ref());

    // Let's model our stream transformation as a function
    //
    //      fn(Source<u8>) -> Source<u32>
    //
    fn flow(mut inp: impl Source<u8>) -> impl Source<u32> {
        // Create a buffer wrapping a byte array for u32 integers.
        let mut buffer = Buffer::from_copy(0u32.to_be_bytes());

        // We need to keep the buffer between invocations,
        // to flush out the buffer data.

        // Create a new source stream from a closure:
        stream::Delegate(move |dest: &mut [u32]| -> IO {
            // The total number of items we places into "dest",
            // which we must also return.
            let mut target = 0;

            loop {
                // If the buffer is full, we get the next item for the
                // destination stream, and clear the buffer for the next loop.
                if buffer.is_full() {
                    // Deserialize a u32.
                    let mut bytes = 0u32.to_be_bytes();
                    bytes.as_mut().copy_from_slice(buffer.as_read());

                    let item = u32::from_be_bytes(bytes);

                    // Try to place in destination.
                    if let Some(cell) = dest.get_mut(target) {
                        *cell = item;
                        target += 1;

                        // After this we restart the loop, so let's clear the buffer
                        // so it has space to read the next series of bytes.
                        buffer.clear();
                    } else {
                        // Destination is full, so we break our loop.
                        break;
                    }
                } else {
                    // Buffer is not full. We will try to read some from the source.
                    if buffer.read(&mut inp)? == 0 {
                        // The source is done.
                        //
                        // In a better implementation we could make it an error if there
                        // are still data in the buffer (!buffer.is_empty()), but for
                        // now we let them disappear.
                        break;
                    } else {
                        // We read some bytes into the empty buffer.
                        // Let the loop go on.
                    }
                }
            }

            // In any case, we return the number of items we managed to place
            // into the destination slice:
            Ok(target)
        })
    }

    // Now we can turn our byte source to a source of u32:
    let u32_source = flow(byte_source);

    // Let's read it into an array of u32:
    let mut u32_arr = [0u32; 8];
    // Through a tiny little buffer
    let num_ints = Buffer::from_copy([0u32; 1]).transfuse(
        // this is already a Source<u32>, as flow() returns it
        u32_source,
        // &mut [T] does *not* implement Sink<T>, but Buffer<T> *does* (under conditions);
        // so we wrap it in a buffer:
        Buffer::from_copy(&mut u32_arr),
    )?;

    // We should have gotten 4 u32 numbers from 16 u8 numbers:
    assert_eq!(num_ints, 4); // This number depends on the source, because u32_arr has 8 places

    // We should also have gotten then right numbers into the array:
    assert_eq!(
        u32_arr,
        [
            0x00010203,
            0x04050607,
            0x08090a0b,
            0x0c0d0e0f,
            0x00000000, // The last four spots are left unwritten, and thus on the default value
            0x00000000,
            0x00000000,
            0x00000000u32
        ]
    );

    // This is an interesting application of buffers and streams, and shows how we can
    // approach streams a-la Akka.
    //
    // The implementation of the flow() function reveals patterns that can be added
    // as an extension to this crate. This work is left for the future.
    //
    // This is just one use case of Buffers and Streams. Check out the rest of the
    // examples for other ideas and different applications.

    Ok(())
}
