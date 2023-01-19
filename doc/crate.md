Bio is a crate for buffered, streaming, I/O transformations.

This crate provides the building blocks and a useful abstraction for
building processing pipelines for I/O.

# `Buffer`

The [`Buffer`] structure provides a simple wrapper for keeping track of
byte-slice usage, during streaming I/O. It is essentially a byte slice with an
extra pair of pointers.