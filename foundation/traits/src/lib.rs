#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[cfg(kani)]
extern crate kani;

pub trait Encoder {
    fn encode(&self, buf: &mut [u8]) -> usize;
}

/// Two-stage zero-copy/alloc decoder trait.
///
/// `Decode` models a two-stage parsing strategy for a buffer holding bytes.
///  1. `populate` reads the bytes from some buffer `buf` and performs validation/parse pass on the borrowed
///     `buf` and returns a reference to buf encapsulated by type `Span` with lifetime `'a`. This span must
///     not allocate; return `Err` of type `Error` for validation/parsing errors.
///  2. `own` consumes the Self (i.e., borrowed `Span`) and returns an owned `Owned` value. `own` is allowed to allocate;
///     callers should call it when they need ownership i.e., lazily own. Implmenentations must copy or clone
///     so the returned value is not constrained by the lifetime `'a`.
///
/// # Implementing `Decode`
///
/// Implementors typically set `type Span = Self`, making the `Span` type the implementor of the trait.
/// `populate` acts as a validating constructor that returns the borrowed `Span`; `own` is then used to
/// perform the allocation into the `Owned` form.
///
pub trait Decode<'a> {
    /// The type that the caller holds ownership of.
    type Owned;
    /// The type returned in the event of a validation/parse error.
    type Error: core::fmt::Debug;
    /// The type that holds the reference to `buf`.
    type Span: 'a + Sized;
    /// Performs the validation/parsing of `buf` and returns Ok(Span) or Err(Error).
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error>;

    /// Consumes `Self` by copying/cloning into `Owned`.
    fn own(self) -> Self::Owned;
}
