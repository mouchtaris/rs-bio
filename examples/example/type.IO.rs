use bio::IO;

#[test]
fn example() -> IO<()> {
    fn one_io() -> IO {
        Ok(1)
    }
    fn two_io() -> IO {
        Ok(2)
    }
    fn add_io() -> IO {
        Ok(one_io()? + two_io()?)
    }

    assert_eq!(add_io().ok(), Some(3));

    Ok(())
}
