use {
    super::{
        stream::{
            Source,
            Sink,
            IntoSource,
        },
    },
};

struct Element;

#[test]
fn slice_is_into_source() {
    [Element].as_slice().into_source();
}

#[test]
fn source_is_object_safe() {
    let _: &dyn Source<Element> =
        &mut [Element].as_ref();
}

#[test]
fn sink_is_object_safe() {
    let _: &dyn Sink<Element> =
        &mut [Element].as_mut();
}