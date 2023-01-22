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

    // Calculate total bytes in files in a traditional way (mirror the approach
    // we'll take with bio Buffers):
    let mut dest = Vec::new();
    let sum0 = PATHS.iter().copied().fold(Ok(0), |sum, path| -> IO {
        let mut file = fs::File::open(path)?;
        let len = std::io::copy(&mut file, &mut dest)? as usize;
        Ok(sum? + len)
    })?;
    assert_eq!(sum0, dest.len());

    // Calculate total bytes in files in a bio way:
    let mut dest = bio::stream::Write(Vec::new()); // The buffer that serves as destination
    let mut buf = bio::Buffer::from_copy([0u8; 128]); // The buffer that serves as intermediate

    // `transfuse()` will also return the number of items moved, so we can as well count the
    // total length in this loop:
    let sum1 = PATHS.iter().copied().fold(Ok(0), |sum, path| -> IO {
        let file = fs::File::open(path)?;
        let source = bio::stream::Read(file);
        let len = buf.transfuse(source, &mut dest)?;
        Ok(sum? + len)
    })?;
    assert_eq!(sum1, dest.0.len());

    // We've gotten the same number of bytes from both:
    assert_eq!(sum0, sum1);

    // This is just one use case of Buffers and Streams, specifically one where
    // T=u8, and the whole `bio` system becomes a more flexible interface for std::io.
    // Check out the rest of the example for cases when we're working with items
    // that are not bytes.

    Ok(())
}
