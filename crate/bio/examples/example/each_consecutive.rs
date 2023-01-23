#[cfg(test)]
use bio::*;

#[test]
fn example() -> IO<()> {
    // In this example we will make use of the EachConsecutive flow.

    // EachConsecutive transforms a Source<T> into a Source<Buffer<[T]>>.

    // It gets constructed with a source Source and a Buffer.
    // The buffer dictates how many items of the old source each consecutive item
    // of the new source will contain by its len().

    // Create a source from an io::Read:
    let source = stream::Read([1, 2, 3, 4, 5u8].as_ref());
    // Define the window buffer to use in EachConsecutive
    let window = Buffer::from_copy([0u8; 3]);

    // Create a sink from a buffer, to inspect the resulting elements:
    let mut sink = Buffer::from_copy([window; 3]);
    // Finally, create an each-consecutive source, wrapping the original source
    // and buffering items in "window":
    let mut source = flow::EachConsecutive::new(source, window);

    // Read as many items as source can give or sink can take:
    let read = sink.read(&mut source)?;

    // This will be 2 items, because after that the source gets depleted:
    assert_eq!(read, 2);

    // The first item is a buffer with the first three elements of source:
    assert_eq!(sink.as_read()[0].as_read(), [1, 2, 3]);

    // The second item is a half-filled buffer, with the remaining elements of source:
    assert_eq!(sink.as_read()[1].as_read(), [4, 5]);

    // The third item of sink is complete unwritten-to:
    assert_eq!(sink.as_read().get(2), None);

    Ok(())
}
