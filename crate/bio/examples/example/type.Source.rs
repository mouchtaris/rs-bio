#[cfg(test)]
use bio::{
    stream,
    Source,
    IO,
};

#[test]
fn example() -> IO<()> {
    // Create a source from a traditional io::Read:
    let mut source = stream::Read([1, 2, 3u8].as_ref());

    // Read from the source into a &[u8] slice:
    let mut dest = [0u8; 2];
    let read = source.source(&mut dest)?;

    // 2 bytes read, as much as destination takes:
    assert_eq!(read, 2);

    // The first two bytes of source are in dest:
    assert_eq!(dest, [1, 2]);

    Ok(())
}
