#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
#[cfg(kani)]
extern crate kani;

pub mod prelude;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "rustls")]
pub mod rustls;
pub mod providers;

use core::time::Duration;

/// `AsyncTransport` is a low-level I/O abstraction that handles reading and
/// writing raw bytes. Implementors wrap a runtime-specific stream (tokio,
/// smol, embassy, etc.) and expose `read`/`write` as `async fn`. Timeout
/// setters remain synchronous as they only configure subsequent I/O and do
/// not perform any.
#[allow(async_fn_in_trait)]
pub trait AsyncTransport {
    type Error: core::fmt::Debug;
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Self::Error>;
    fn set_write_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Self::Error>;
}

/// `AsyncSender` encodes a message struct `M` and uses the transport's
/// `write` to send it.
#[allow(async_fn_in_trait)]
pub trait AsyncSender<M, T: AsyncTransport> {
    type Error: core::fmt::Debug;
    async fn send(&mut self, msg: M) -> Result<(), Self::Error>;
}

/// `AsyncReceiver` reads incoming traffic into the session's internal buffer
/// and exposes a zero-copy span `Output<'_>` that borrows directly from it.
/// Receive is split into two phases — `receive()` populates the internal
/// buffer, `output()` returns the borrow — because returning a borrow from an
/// `async fn` requires lending-future support that is not yet stable in 2026.
/// `output()` is fallible because parsing the buffered bytes can fail even
/// after a successful read.
#[allow(async_fn_in_trait)]
pub trait AsyncReceiver<T: AsyncTransport> {
    type Error: core::fmt::Debug;
    type Output<'a>
    where
        Self: 'a;
    async fn receive(&mut self) -> Result<(), Self::Error>;
    fn output(&self) -> Result<Self::Output<'_>, Self::Error>;
}