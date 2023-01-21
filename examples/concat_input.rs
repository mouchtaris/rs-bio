fn main() {}

#[cfg(test)]
use {
    bio::{
        self,
        IO,
    },
    std::fs,
};

#[test]
fn example() -> IO<()> {
    // We are going to read a series of files to a destination Vec<u8>.
    // We'll double check that the number of bytes in the destination is
    // the sum of bytes if we loaded each file separately.

    const PATHS: &[&str] = &[
        "doc/sample_text/BufferedCarbon.md",
        "doc/sample_text/Bufferwards.md",
        "doc/sample_text/Comedy.md",
        "doc/sample_text/GameOfBuffers.md",
        "doc/sample_text/HarukiBufferami.md",
        "doc/sample_text/TheBufftrix.md",
        "doc/sample_text/TomBuffins.md",
    ];

    // (We skip optimisations in the control flow in favour of clarity)

    // Calculate total bytes in files in the traditional way:
    let sum0 = PATHS.iter().copied().fold(Ok(0), |sum, path| -> IO {
        let content = fs::read_to_string(path)?;
        let len = content.as_bytes().len();
        Ok(sum? + len)
    })?;

    // Now instead of loading each file into a separate allocated buffer,
    // we will stream-read it directly into one destination allocated buffer.

    // Note that "_copy" in those constructors means that the item type (u8) is `Copy`.
    // This is used to specialize the buffer into using the optimised [T]::* operations
    // when T: Copy.

    let mut dest = bio::stream::Write(Vec::new()); // The buffer that serves as destination
    let mut buf = bio::Buffer::from_copy([0u8; 128]); // The buffer that serves as intermediate

    // `transfuse()` will anyway return the number of items moved, so we can already count the
    // total length in this loop:
    let sum1 = PATHS.iter().copied().fold(Ok(0), |sum, path| -> IO {
        let file = fs::File::open(path)?;
        let source = bio::stream::Read(file);
        let len = buf.transfuse(source, &mut dest)?;
        Ok(sum? + len)
    })?;

    // But the total number of bytes should also be in the destination buffer:
    let sum2 = dest.0.len();

    assert_eq!(sum0, sum1);
    assert_eq!(sum1, sum2);

    // This might seem like a more verbose approach, but consider the traditional way
    // of appending every input stream to the same output stream.
    //
    // Essentially, this approach serves as a replacement for a missing
    //
    //     std::io::read_all_into(io::Read, io::Write)

    // This is just one use case of Buffers and Streams, specifically one where
    // T=u8, and the whole `bio` system becomes a more flexible interface for std::io.
    // Check out the rest of the examples for cases when we're working with items
    // that are not bytes.

    Ok(())
}
