A generalisation of [`std::io::Read`] for any type `T`

A [`Source`] is a more general version of [`std::io::Read`], dealing with
items of type `T` instead of `u8`.

# Usage

Using a source mirrors the way one might use [`std::io::Read`].

The one particular difference is that it accepts a slice of `T` instead of `u8`.

# Errors

#[include_str!("LOL")]

# Example