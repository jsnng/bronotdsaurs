pub mod error;
pub mod login;
pub mod observer;
pub mod prelogin;
pub mod prelude;
pub mod sql_batch;
pub mod state;
pub mod timer;
pub mod traits;
/// Special transport adaptor is required for TDS 7.x only as
/// PreLogin packets wraps the TLS handshake bytes.
#[cfg(all(not(feature = "tds8.0"), feature = "tls"))]
pub mod adaptor;
#[cfg(feature = "rustls")]
pub use transport::rustls;

use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[cfg(kani)]
const KANI_BUFFER_SIZE: usize = 64;

#[cfg(not(kani))]
const BUFFER_SIZE: usize = MAX_TDS_PACKET_BYTES;
#[cfg(kani)]
const BUFFER_SIZE: usize = KANI_BUFFER_SIZE;

/// A resetable linear sliding window buffer.
/// `head` and `tail` marks the start and end of unconsumed data respectively.
/// data is written to `[tail..]` and consumed from `[head..tail]`.
/// `reset()` is used to reset the buffer.
#[derive(Debug)]
pub struct SessionBuffer {
    buffer: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    size: Option<usize>,
}

impl Default for SessionBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; BUFFER_SIZE],
            head: 0,
            tail: 0,
            size: None,
        }
    }
}

impl SessionBuffer {
    /// return the slice of the buffer with unconsumed data: `buffer[head..tail]`
    #[inline(always)]
    pub fn readable(&self) -> &[u8] {
        &self.buffer[self.head..self.tail]
    }

    /// return the slice of the buffer available for writing: `buffer[tail..]`:
    #[inline(always)]
    pub fn writeable(&mut self) -> &mut [u8] {
        let max_size = self.buffer_size();
        &mut self.buffer[self.tail..max_size]
    }

    /// move head cursor by `n`. Errors if the move is to exceed the tail cursor.
    #[inline(always)]
    pub fn head(&mut self, n: usize) -> Result<(), SessionError> {
        if self.head + n > self.tail {
            return Err(SessionError::BufferIndexOutOfBoundsError(
                #[cfg(not(kani))]
                alloc::format!(
                    "move self.head index invalid {}, self.tail is at {}",
                    self.head + n,
                    self.tail
                ),
                #[cfg(kani)]
                ("").to_string(),
            ));
        }
        self.head += n;
        Ok(())
    }

    /// move tail cursor by `n`. Errors if the move exceeds `size` if `size.is_some()` or `BUFFER_SIZE`
    #[inline(always)]
    pub fn tail(&mut self, n: usize) -> Result<(), SessionError> {
        let max_size = self.buffer_size();
        if self.tail + n > max_size {
            return Err(SessionError::BufferIndexOutOfBoundsError(
                #[cfg(not(kani))]
                alloc::format!(
                    "move self.tail index invalid {}. Exceeds self.size {}",
                    self.tail + n,
                    max_size
                ),
                #[cfg(kani)]
                ("").to_string(),
            ));
        }
        self.tail += n;
        Ok(())
    }

    /// return the number of unconsumed bytes in the buffer
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    /// return true if number of consumable bytes is 0
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// move the head and tail cursors to 0.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    /// changes the maximum size of the writeable section to `size`. Errors if `size > BUFFER_SIZE`.
    pub fn set_buffer_maximum_size(&mut self, size: usize) -> Result<(), SessionError> {
        if size > BUFFER_SIZE {
            return Err(SessionError::RequestedPacketSizeTooLarge);
        }
        self.size = Some(size);
        Ok(())
    }

    /// the current maximum writeable size: the negotiated `size`, or `BUFFER_SIZE` if unset.
    #[inline(always)]
    pub fn buffer_size(&self) -> usize {
        self.size.unwrap_or(BUFFER_SIZE)
    }
}

impl core::fmt::Display for SessionBuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let buffer = &self.buffer[..self.tail];
        writeln!(f, "self.head = {}, self.tail = {}\n", self.head, self.tail)?;
        for (i, chunk) in buffer.chunks(16).enumerate() {
            for b in chunk {
                write!(f, "{:02x} ", b)?;
            }
            let padding = 16 - chunk.len();
            if padding > 0 {
                write!(f, "{:width$}", "", width = padding * 3)?;
            }
            writeln!(f, "{:03}:", i * 16)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Session<S, T, O> {
    pub(super) stream: T,
    pub(super) observer: O,
    pub(super) timers: Timers,
    pub(super) buffer: SessionBuffer,
    pub(super) state: S,
}

impl<S: Default, T, O> Session<S, T, O> {
    pub fn new(stream: T, observer: O) -> Self {
        Self {
            stream,
            observer,
            timers: Timers::default(),
            buffer: SessionBuffer::default(),
            state: S::default(),
        }
    }
}

impl<S, T, O: Observer<Event>> Session<S, T, O> {
    pub fn notify(&mut self, event: Event) {
        self.observer.on(&event);
    }
}

impl<S, T, O, M> AsyncSender<M, T> for Session<S, T, O>
where
    T: AsyncTransport,
    O: Observer<Event>,
    M: MessageEncoder<Error = EncodeError>,
    M::Header: Default,
{
    type Error = SessionError;
    #[inline]
    async fn send(&mut self, msg: M) -> Result<(), Self::Error> {
        self.buffer.reset();
        let len = msg.oneshot(&mut self.buffer, &mut M::Header::default())?;
        self.buffer.tail(len)?;

        self.notify(Event::BytesSent {
            heading: core::any::type_name::<M>(),
            len,
        });

        let mut offset = 0;
        while offset < len {
            let n = self
                .stream
                .write(&self.buffer.readable()[offset..len])
                .await
                .map_err(|_| SessionError::transport_write_error())?;
            if n == 0 {
                return Err(SessionError::ServerClosedTransportConnection);
            }
            offset += n;
        }
        self.buffer.reset();
        Ok(())
    }
}

#[cfg(feature = "std")]
impl From<String> for SessionError {
    fn from(val: String) -> Self {
        SessionError::MappedError(val)
    }
}

#[cfg(kani)]
#[kani::proof]
fn session_buffer_writable_cant_exceed_max_tds_packet_size() {
    let mut buffer = SessionBuffer::default();
    let tail: usize = kani::any();
    kani::assume(tail <= BUFFER_SIZE);
    buffer.tail(tail).unwrap();

    let use_max_size: bool = kani::any();
    if use_max_size {
        let size: usize = kani::any();
        kani::assume(size <= BUFFER_SIZE);
        kani::assume(tail <= size);
        buffer.set_buffer_maximum_size(size).unwrap();
    }
    assert!(buffer.writeable().len() <= BUFFER_SIZE)
}
