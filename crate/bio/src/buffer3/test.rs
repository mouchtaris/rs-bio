use super::*;

#[test]
fn buffer_source() -> IO<()> {
    let mut s0 = Buffer::from_copy([1, 2, 3, 4, 5]).as_source();

    let mut dest = [0u8; 3];
    s0.source(&mut dest)?;
    assert_eq!(dest, [1, 2, 3]);

    let mut dest = [0u8; 3];
    s0.source(&mut dest)?;
    assert_eq!(dest, [4, 5, 0]);

    Ok(())
}
#[test]
fn buffer_sink() -> IO<()> {
    let mut s0 = Buffer::from_copy([0u8; 5]);

    s0.sink(&[1, 2])?;
    assert_eq!(s0.as_read(), [1, 2]);

    s0.sink(&[3, 4])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4]);

    s0.sink(&[5, 6])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4, 5]);

    s0.sink(&[7, 8])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4, 5]);

    Ok(())
}
#[test]
fn buffer_interop() -> IO<()> {
    let mut source = Buffer::from_copy([1, 2, 3, 4, 5u8]).as_source();
    let mut sink = Buffer::from_copy([0u8; 4]);
    let mut buf = Buffer::from_copy([0u8; 3]);

    buf.read(&mut source)?;
    assert_eq!(buf.as_read(), [1, 2, 3]);
    assert_eq!(source.as_read(), [4, 5]);

    buf.write(&mut sink)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);

    // Buffer is stack at avail() == free() == 0

    buf.read(&mut source)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(source.as_read(), [4, 5]);

    buf.write(&mut sink)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);

    Ok(())
}
#[test]
fn buffer_transfuse_short_sink() -> IO<()> {
    let mut source = Buffer::from_copy([1, 2, 3, 4, 5u8]).as_source();
    let mut sink = Buffer::from_copy([0u8; 4]);

    let mut buf = Buffer::from_copy([0u8; 3]);

    let n = buf.transfuse(&mut source, &mut sink)?;

    assert_eq!(n, 4);
    assert_eq!(source.as_read(), []); // Buffered data lost
    assert_eq!(sink.as_read(), [1, 2, 3, 4]);
    assert_eq!(buf.as_read(), [5]); // Buffered data pending
    Ok(())
}
#[test]
fn buffer_transfuse_short_source() -> IO<()> {
    let mut source = Buffer::from_copy([1, 2, 3u8]).as_source();
    let mut sink = Buffer::from_copy([0u8; 4]);

    let mut buf = Buffer::from_copy([0u8; 3]);

    let n = buf.transfuse(&mut source, &mut sink)?;

    assert_eq!(n, 3);
    assert_eq!(source.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);
    assert_eq!(buf.as_read(), []);
    Ok(())
}
#[test]
fn cbuffer_source() -> IO<()> {
    let mut s0 = Buffer::from_clone([1, 2, 3, 4, 5]).as_source();

    let mut dest = [0u8; 3];
    s0.source(&mut dest)?;
    assert_eq!(dest, [1, 2, 3]);

    let mut dest = [0u8; 3];
    s0.source(&mut dest)?;
    assert_eq!(dest, [4, 5, 0]);

    Ok(())
}
#[test]
fn cbuffer_sink() -> IO<()> {
    let mut s0 = Buffer::from_clone([0u8; 5]);

    s0.sink(&[1, 2])?;
    assert_eq!(s0.as_read(), [1, 2]);

    s0.sink(&[3, 4])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4]);

    s0.sink(&[5, 6])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4, 5]);

    s0.sink(&[7, 8])?;
    assert_eq!(s0.as_read(), [1, 2, 3, 4, 5]);

    Ok(())
}
#[test]
fn cbuffer_interop() -> IO<()> {
    let mut source = Buffer::from_clone([1, 2, 3, 4, 5u8]).as_source();
    let mut sink = Buffer::from_clone([0u8; 4]);
    let mut buf = Buffer::from_clone([0u8; 3]);

    buf.read(&mut source)?;
    assert_eq!(buf.as_read(), [1, 2, 3]);
    assert_eq!(source.as_read(), [4, 5]);

    buf.write(&mut sink)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);

    // Buffer is stack at avail() == free() == 0

    buf.read(&mut source)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(source.as_read(), [4, 5]);

    buf.write(&mut sink)?;
    assert_eq!(buf.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);

    Ok(())
}
#[test]
fn cbuffer_transfuse_short_sink() -> IO<()> {
    let mut source = Buffer::from_clone([1, 2, 3, 4, 5u8]).as_source();
    let mut sink = Buffer::from_clone([0u8; 4]);

    let mut buf = Buffer::from_clone([0u8; 3]);

    let n = buf.transfuse(&mut source, &mut sink)?;

    assert_eq!(n, 4);
    assert_eq!(source.as_read(), []); // Buffered data lost
    assert_eq!(sink.as_read(), [1, 2, 3, 4]);
    assert_eq!(buf.as_read(), [5]); // Buffered data pending
    Ok(())
}
#[test]
fn cbuffer_transfuse_short_source() -> IO<()> {
    let mut source = Buffer::from_clone([1, 2, 3u8]).as_source();
    let mut sink = Buffer::from_clone([0u8; 4]);

    let mut buf = Buffer::from_clone([0u8; 3]);

    let n = buf.transfuse(&mut source, &mut sink)?;

    assert_eq!(n, 3);
    assert_eq!(source.as_read(), []);
    assert_eq!(sink.as_read(), [1, 2, 3]);
    assert_eq!(buf.as_read(), []);
    Ok(())
}
#[test]
fn buffer_io() -> IO<()> {
    let read = [0, 1, 2, 3, 4, 5u8].as_ref();
    let mut source = stream::Read(read.as_ref());

    let mut write = [0u8; 19];
    let mut sink = stream::Write(write.as_mut());

    let tr = Buffer::from_copy([0u8; 18]).transfuse(&mut source, &mut sink)?;

    assert_eq!(tr, 6);
    assert_eq!(write.as_mut()[0..7], [0, 1, 2, 3, 4, 5, 0]);
    Ok(())
}
