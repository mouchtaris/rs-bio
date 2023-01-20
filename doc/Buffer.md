A managed byte-slice that keeps track of written and read areas.

A `Buffer` is essentially a byte slice with 2 extra pointers (expressed as indices):

- `position`: the offset into the slice that can be read from; `position` advances with
   every "read" from the buffer, and can go up to `limit`,
- `limit`: the offset into the slice that can be written to; `limit` serves as both the
   ending-index for reads and the starting-index for writes; it advances with every "write"
   to the buffer,

A couple of synthetic properties arise from these pointers:

- `available` is the number of *available-for-reading* bytes, defined as the difference between
   `limit` and `position`,
- `free` is the number of *free-to-write* bytes, defined as the difference between the length
   of the backing slice and `limit`.

As a convenience [`Self::is_empty()`] means `available() == 0` and [`Self::is_full()`] means
`free() == 0`.

In a schematic form:

```text
    Slice: [xxxxxxxxxxxxxxxoooooooooooooooooo................]
            ^              ^                 ^              |
            |              |                 |              |
         index(0)          |                 |        index(len-1)
            |          position            limit            |
            |              |                 |              |
            |--------------|                 |              |
          This segment has been              |              |
          written to and read from           |              |
                           |                 |              |
                           |---AVAILABLE-----|              |
                        This segment has been written to    |
                        but not yet read from               |
                                             |              |
                                             |---FREE-------|
                                            This segment is free to be written to
```

The invariant rules that come for this are:

```text
    0 <= position <= limit <= len       (1)
    0 <= available         <= len       (2)
    0 <= free              <= len       (3)
    is_empty  ===  available == 0       (4)
    is_full   ===  free == 0            (5)
```

Note that both offsets point to "the next available" spot, and can both reach `len`.
These details, however, are exactly the abstraction value of this type, and a user would never
have to keep in mind.

A buffer generally operates by wrapping a `D` that is `AsRef<[u8]>` and/or `AsMut<[u8]>`.
This requirement is not imposed on creation, but operations (associated functions) will be
enabled gradually when these requirements are met.

As the name denotes, the `Buffer` concept is lend from JVM's [`java.nio.ByteBuffer`]. For
a more in depth explanation of the concept, you can refer to the linked API. One small
difference from the JVM API is that this `Buffer` does not require `flip()`ing between
writes and reads, because it offers less capabilities that the JVM's buffer, and therefore
flipping becomes redundant.


# Usage

## Creating a `Buffer`

Creating a buffer happens through [`Self::new()`], which accepts a backing storage in any form
(owned or any-way-borrowed).

```rust
use bio::Buffer;

const N: usize = 11;

fn impl_as_ref_u8() -> impl AsRef<[u8]> {
    "Hello, BIO!"
}

// Choose your backing storage
let store0 = impl_as_ref_u8();      // an opaque AsRef<[u8]>
let mut store1 = vec![0u8; N];      // A vector of u8
let mut store2 = [0u8; N];          // A static array of u8

// Choose your wrapping style
let buf1 = Buffer::new(&store0);     // borrowed, will enable only read-only operations
let buf2 = Buffer::new(&mut store1); // mutably borrowed, will enable all operations
let buf3 = Buffer::new(store2);      // owned, will enable all operations

// Buffers ready
assert_eq!(buf1.free(), N);
assert_eq!(buf2.free(), N);
assert_eq!(buf3.free(), N);
```

## Including in other types

If you need to include a buffer in another type, you have two choices:

- propagate the type variable `D` to your type's type parameters, or
- use [`OwnedBuffer`] or write it yourself as `Buffer<Vec<u8>>`.

```rust
use bio::Buffer;
use bio::OwnedBuffer;

// Reading over a read-only buffer:
struct ReadingSlowly<'a>(pub Buffer<&'a str>);

// Own it completely:
struct MyByteBuffer(OwnedBuffer);

// Or own it yourself:
struct MyString(Buffer<String>);
```

## Reclaiming backing storage

You can at any point lose the buffer and get back `D` by calling [`Self::to_inner()`]:


## Performing I/O

The main point and usage of the `Buffer` is to read from and write to I/O streams.

`Buffer` provides the [`Self::read()`] and [`Self::write()`] functions, which accept an I/O
object and delegate the reading/writing to that. What `Buffer` takes care of is passing the
appropriate part of the backing slice to I/O. This is most handy when there are multiple,
possibly multiplexed, I/O operations, which are incomplete. Using the buffer contract there
is no book-keeping that needs to happen from the user.

### Reading

You can *read-into* the buffer from any [`io::Read`] object. This operation can be called
repeatedly, until the buffer [`Self::is_full()`]. It can still be called when the buffer iso
full, but it will consistently read 0 extra bytes, so care should be taken.

```rust
# fn main() -> std::io::Result<()> { use bio::Buffer;

let mut buffer = Buffer::new([0u8; 3]);
let mut source = [0u8, 1, 2].as_ref();
buffer.read(&mut source)?;

assert_eq!(buffer.to_inner(), [0, 1, 2]);

# Ok (()) }
```

### Writing

You can *write-from* the buffer to any [`io::Write`] object. This operation can be called
repeatedly, until the buffer [`Self::is_empty()`]. It can still be called when the buffer
is empty, but it will consistently write 0 extra bytes, so care should be taken.

```rust
# fn main() -> std::io::Result<()> { use bio::Buffer;

let mut buffer = Buffer::source([0u8, 1, 2]);
let mut dest = [0u8; 5];
buffer.write(&mut dest.as_mut())?;

assert_eq!(dest, [0, 1, 2, 0, 0]);
# Ok (()) }
```

### Between writes and reads

If you are interleaving reads and writes, `position` and `limit` are progressively moving
toward the `len` of the backing slice. This causes the [`Self::free()`] space to gradually be
less and less, down to `0`. Eventually, every read and write will be no-op, because
```text
    position == limit == len   =>
    available == free == 0
```

In order to avoid this situation, and to keep the buffer space maximally utilized at all times,
between I/Os one should call [`Self::compact()`], which will copy internally all areas of the
buffer back over the completely consumed areas of it (`0..position`).

This is a very cheap operation, as it can happen with [`slice::copy_within()`] of byte slices.

```rust
# fn main() -> std::io::Result<()> { use bio::Buffer;

let mut buffer = Buffer::new([0u8; 2]);
let mut source: &[u8] = &[1u8, 2, 3, 4];
let mut dest = [0u8; 3];

// Read 2 bytes (buffer capacity) from source
buffer.read(&mut source)?;
assert!(buffer.is_full());

// Write 2 bytes (buffer contents) to dest
buffer.write(&mut dest.as_mut())?;
assert!(buffer.is_empty());

// Now dest contains the first two bytes of source
assert_eq!(dest, [1, 2, 0]);

// Further reading or writing from the buffer is a no-op
// because position and limit are maxed out
buffer.read(&mut source)?;
buffer.write(&mut &mut dest[2..])?;
// Destination still lacks the final byte
assert_eq!(dest, [1, 2, 0]);

// Solution: compact the buffer
buffer.compact();
buffer.read(&mut source)?;
buffer.write(&mut &mut dest[2..])?;
// Destination now contains all source bytes
assert_eq!(dest, [1, 2, 3]);
   
// NOTE the manual book-keeping for the dest slice!
// That's exactly what we have the buffer for. See next section, also.

# Ok (()) }
```


## Using the buffer as an I/O object

Since `Buffer` wraps `[u8]` values, and references to those implement the I/O traits,
the buffer itself implements I/O as well.

The default `Self::new()` constructor creates a buffer that is initially empty, and is
better suited as a destination buffer (at least at first, until it has data written into it,
which can be later read).

The `Self::source()` constructor creates a buffer that is initially configured directly
for reading the data of the backing storage. In other words, makes the whole storage range
`Self::available()`.

The earlier example can be made more easy by using only buffers:

```rust
# fn main() -> std::io::Result<()> { use bio::Buffer;

let mut buffer = Buffer::new([0u8; 2]);
let mut source = Buffer::source([1u8, 2, 3, 4]);
let mut dest = Buffer::new([0u8; 3]);

// Now it's just too easy
buffer.read(&mut source)?;
buffer.write(&mut dest)?;

// Buffer capacity was 2, so only 2 source bytes in dest:
assert_eq!(dest.as_read(), [1, 2]); // Buffer as slice, see below section.

buffer.compact();
buffer.read(&mut source)?;  // source buffer keeps track where we're reading from
buffer.write(&mut dest)?;   // dest buffer keeps track of where we're writing to

// Now we're done
assert_eq!(dest.to_inner(), [1, 2, 3]);
# Ok (()) }
```


## I/O transfusion

If you have made it thus far, you have undoubtedly noted an emerging pattern for moving data
from an input to an output.

Buffer provides the [`Self::transfuse()`] function, for repeatedly calling the
[`Self::read()`], [`Self::write()`], [`Self::compact()`] cycle.

Note that `transfuse` will stop only when no more bytes can be moved.
There is no way to short-cut this operation. For more sophisticated transfusions, one
should write their own utility.

Make sure to check the function's documentation for how the I/O objects are affected.

```rust
# fn main() -> std::io::Result<()> { use bio::Buffer;
let mut source = Buffer::source([1u8, 2, 3]);
let mut dest = Buffer::new([0u8; 2]);
let mut buffer = Buffer::new([0u8]); // 1 byte buffer!

let transfused = buffer.transfuse(&mut source, &mut dest)?;

assert_eq!(transfused, 2);          // only 2 bytes fit in destination
assert_eq!(dest.as_read(), [1, 2]);
# Ok (()) }
```


## Borrow the buffer as slices

`Buffer` provides views into the backing slice as sub-slices.

These views are a backdoor and bypass the invariant contracts! Using them will not update
the internal pointers, but will also not mess them up, so you can resume using the buffer
after having an indiscreet peek inside. Therefore, these are provided in a
*use-at-your-own-risk* fashion.

The [`Self::available()`] area, which are bytes to be read from, is returned from
[`Self::as_read()`].

The [`Self::free()`] area, which are bytes to be written to, is returned from
[`Self::as_write()`].

```rust
# fn main() -> std::io::Result<()> {
use bio::Buffer;

let mut buffer = Buffer::new([0u8; 3]);

// Initially, available is 0
assert_eq!(buffer.as_read(), &[]);
// ... but free is everything
assert_eq!(buffer.as_write(), &mut [0, 0, 0]);

buffer.read(&mut [1u8, 2].as_ref())?;
// Now there are 2 bytes to be read from
assert_eq!(buffer.as_read(), &[1, 2]);
// ... and one to be written to
assert_eq!(buffer.as_write(), &mut [0]);
# Ok (()) }
```


## Buffer traits

Buffer implements [`AsRef`] and [`AsMut`] to a slice type `[T]`, when the backing storage
`D` implements those traits.

Buffer also implements [`io::Read`] and [`io::Write`], by reading/writing from its
[`Self::as_read()`] and [`Self::as_write()`] areas.


## Buffers within buffers

It should be clear by now that buffers can be the backing storage of buffer. Bufferception.

```rust
use bio::Buffer;

let mut buffer = Buffer::new(Buffer::new(Buffer::new(Buffer::new(vec![0u8; 1]))));
let mut buffer = Buffer::new(Buffer::new(vec![0u8; 1]));
assert_eq!(buffer.available(), 0);
assert_eq!(buffer.free(), 1);
```

Whether this is useful or not is not something asserted in this writing.


# Collaboration with other crates

[`Buffer`] is the building block for [`sio`], the *Streaming I/O* crate. Have a look at that,
for even more funkiness.


# Have a nice day

Thank you for reading. Enjoy `Buffer`!

---
[`sio`]: /crate/sio.html