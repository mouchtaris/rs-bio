A generalisation of [`io::Read`] for any type `T`

A [`Source`] is a more general version of [`io::Read`], dealing with
items of type `T` instead of `u8`.

# Usage

Using a source mirrors the way one might use [`io::Read`].

The one particular difference is that it accepts a slice of `T` instead of `u8`.

# Errors

A source might return any [`io::Error`] that would most appropriately
fit the type of error that has occurred.

## Wrapping [`io::Read`]

When the [`Source`] wraps an [`io::Read`] through a [`stream::Read`], the underlying
actual [`io::Error`]s will be propagated to the [`Source::source`] result.

## Wrapping [`Buffer`]

When a [`Buffer`] is used as a source, there will be no errors returned, ever.

## Flows that implement [`Source`]

Particular [`Flow`]s might return an `io::Error`. The details for each flow should be on
their respective documentation.