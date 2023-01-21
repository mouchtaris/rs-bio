use {
    super::buffer2::*,
    std::io,
};

#[test]
fn useful() -> io::Result<()> {
    let arr = [0, 1, 2u8];

    let data = arr.clone();
    let mut s0 = Read(data.as_ref());

    let mut data = arr.clone();
    let mut n0 = Write(data.as_mut());

    let data = arr.clone();
    let mut b = Copy::new(data);

    b.read(&mut s0)?;
    b.write(&mut n0)?;

    b.read(s0)?;
    b.write(n0)?;

    struct V(u8);
    let arr = || [V(0), V(1), V(2)];
    let mut b = Move::new(arr());

    let read = b.read(arr())?;
    let write = b.write(arr())?;
    assert_eq!(read, 3);
    assert_eq!(write, 3);

    Ok(())
}
