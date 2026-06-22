//! SQL Batch State Transitions
use core::mem::MaybeUninit;

use crate::tds::decoder::stream::{NoContextStep, TokenDecoder};
use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[derive(Debug)]
pub struct QueryResult {
    pub done_token: DoneToken,
    pub errors: Vec<ErrorInfoToken>,
    pub return_status: Option<ReturnStatusToken>,
}

#[derive(Debug)]
pub struct QueryResults {
    pub results: Vec<QueryResult>,
}

#[derive(Debug)]
pub enum LoggedInStateTransition<T, O> {
    Ok {
        session: Session<LoggedInState, T, O>,
        results: QueryResults,
    },
    Error {
        session: Session<LoggedInState, T, O>,
        errors: Vec<ErrorInfoToken>,
    },
}

/// TODO: planned replacement for receive state.
struct DecodeState {
    results: Vec<QueryResult>,
    errors: Vec<ErrorInfoToken>,
    col_metadata: Option<ColMetaDataOwned>,
    return_status: Option<ReturnStatusToken>,
    done_token: Option<DoneToken>,
}

impl DecodeState {
    fn new() -> Self {
        Self {
            results: Vec::new(),
            errors: Vec::with_capacity(4),
            col_metadata: None,
            return_status: None,
            done_token: None,
        }
    }
}

/// A zero-alloc streaming buffer backed by a fixed-size uninitialized array.
struct StreamingBuffer {
    bytes: [MaybeUninit<u8>; 2 * MAX_TDS_PACKET_BYTES],
    head: usize,
    tail: usize,
    eof: bool,
}

impl StreamingBuffer {
    #[inline]
    fn new() -> Self {
        Self {
            bytes: [const { MaybeUninit::uninit() }; 2 * MAX_TDS_PACKET_BYTES],
            head: 0,
            tail: 0,
            eof: false,
        }
    }

    /// Reclaims consumed bytes at the front of the buffer.
    #[inline]
    fn compact(&mut self) {
        if self.head > 0 {
            let remaining = self.tail - self.head;
            self.bytes.copy_within(self.head..self.tail, 0);
            self.head = 0;
            self.tail = remaining;
        }
    }

    /// Reads the next TDS packet from `stream`, strips its 8-byte header, and
    /// writes the payload directly into the buffer.
    async fn fill<T: AsyncTransport>(&mut self, stream: &mut T) -> Result<(), SessionError> {
        const LENGTH: usize = 8;
        let mut header = [0u8; LENGTH];
        let mut idx = 0;
        while idx < LENGTH {
            let n = stream
                .read(&mut header[idx..])
                .await
                .map_err(|_| SessionError::transport_read_error())?;
            if n == 0 {
                return Err(SessionError::ServerClosedTransportConnection);
            }
            idx += n;
        }

        let ty = header[0];
        if ty != SERVER_PACKET_TYPE {
            return Err(SessionError::InvalidPacketType { got: ty });
        }
        
        let status = header[1];
        let length = u16::from_be_bytes([header[2], header[3]]) as usize;
        if length < LENGTH {
            return Err(SessionError::PartialRead);
        }

        let payload_length = length - LENGTH;

        let mut reading = 0;
        while reading < payload_length {
            // SAFETY: self.tail + payload_length <= 2 * MAX_TDS_PACKET_BYTES,
            // and we are writing into this range (initializing it).
            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                    self.bytes[self.tail + reading..self.tail + payload_length].as_mut_ptr() as *mut u8,
                    payload_length - reading,
                )
            };
            let n = stream
                .read(dst)
                .await
                .map_err(|_| SessionError::transport_read_error())?;
            if n == 0 {
                return Err(SessionError::ServerClosedTransportConnection);
            }
            reading += n;
        }
        self.tail += payload_length;

        if (status & MessageStateStatus::EndOfMessage) != 0 {
            self.eof = true;
        }
        Ok(())
    }
}

impl<T: AsyncTransport, O: Observer<Event>> Session<LoggedInState, T, O> {
    #[inline]
    async fn send_and_receive<Msg, M, F>(
        mut self,
        msg: Msg,
        attention: Attention,
        on_col_metadata: M,
        on_row: F,
    ) -> Result<LoggedInStateTransition<T, O>, SessionError>
    where
        Msg: MessageEncoder<Error = EncodeError>,
        Msg::Header: Default,
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        self.send(msg).await?;
        let results = self.receive(attention, on_col_metadata, on_row).await?;
        let errors: Vec<ErrorInfoToken> = results.results.iter()
            .flat_map(|r| r.errors.iter().cloned())
            .collect();
        if !errors.is_empty() {
            Ok(LoggedInStateTransition::Error {
                session: self,
                errors,
            })
        } else {
            Ok(LoggedInStateTransition::Ok {
                session: self,
                results,
            })
        }
    }

    #[inline]
    pub async fn query<M, F>(
        self,
        sql_batch: SQLBatch,
        attention: Attention,
        on_col_metadata: M,
        on_row: F,
    ) -> Result<LoggedInStateTransition<T, O>, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        self.send_and_receive(sql_batch, attention, on_col_metadata, on_row).await
    }

    #[inline]
    pub async fn execute<M, F>(
        self,
        rpc: RPCReqBatch,
        attention: Attention,
        on_col_metadata: M,
        on_row: F,
    ) -> Result<LoggedInStateTransition<T, O>, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        self.send_and_receive(rpc, attention, on_col_metadata, on_row).await
    }

    /// Decodes the TDS response stream, drains() row tokens when col_metadata is received via callbacks.
    #[inline]
    pub async fn receive<M, F>(&mut self, attention: Attention, mut on_col_metadata: M, mut on_row: F) -> Result<QueryResults, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        let mut buf = StreamingBuffer::new();
        let mut results: Vec<QueryResult> = Vec::new();
        let mut errors: Vec<ErrorInfoToken> = Vec::with_capacity(4);
        let mut col_metadata_owned: Option<ColMetaDataOwned> = None;
        let mut return_status: Option<ReturnStatusToken> = None;
        let mut done_token: Option<DoneToken> = None;

        'outer: loop {
            // Fast path: drain rows while we have column metadata
            if let Some(ref col_metadata) = col_metadata_owned {
                let col_metadata_span = col_metadata.borrow();
                // SAFETY: buf.head..buf.tail is initialized by prior fill() calls.
                // Raw pointer avoids borrowing buf so buf.head can be mutated below.
                let decoder = TokenDecoder::resume(unsafe {
                    let ptr = buf.bytes.as_ptr().add(buf.head) as *const u8;
                    core::slice::from_raw_parts(ptr, buf.tail - buf.head)
                }, col_metadata_span);
                let (done, consumed) = decoder.drain(|row| on_row(col_metadata, row));
                buf.head += consumed;
                if let Some(span) = done {
                    if span.is_final() {
                        done_token = Some(span.own());
                        break 'outer;
                    }
                    results.push(QueryResult {
                        done_token: span.own(),
                        errors: core::mem::take(&mut errors),
                        return_status: return_status.take(),
                    });
                    col_metadata_owned = None;
                    continue 'outer;
                }
                // drain stalled — peek to decide: need more data or unknown token?
                let stalled_on = if buf.head < buf.tail {
                    Some(unsafe { *buf.bytes[buf.head].as_ptr() })
                } else {
                    None
                };
                match stalled_on {
                    Some(0xd1) | Some(0xd2) => {
                        // incomplete row — need more data
                        if buf.eof { break 'outer; }
                        buf.compact();
                        buf.fill(&mut self.stream).await?;
                        continue 'outer;
                    }
                    Some(b) if b >= 0xfd => {
                        // incomplete done token — need more data
                        if buf.eof { break 'outer; }
                        buf.compact();
                        buf.fill(&mut self.stream).await?;
                        continue 'outer;
                    }
                    None => {
                        // buffer exhausted — need more data
                        if buf.eof { break 'outer; }
                        buf.compact();
                        buf.fill(&mut self.stream).await?;
                        continue 'outer;
                    }
                    _ => {
                        // unknown token — drop context, fall through to NoContext
                        col_metadata_owned = None;
                    }
                }
            }

            // Slow path: advance through non-row tokens until we get new col_metadata
            let mut head = buf.head;
            // SAFETY: head..buf.tail is initialized by prior fill() calls.
            let mut decoder = TokenDecoder::new(unsafe {
                let ptr = buf.bytes.as_ptr().add(head) as *const u8;
                core::slice::from_raw_parts(ptr, buf.tail - head)
            });
            loop {
                match decoder.advance() {
                    #[cfg(feature = "tds7.4")]
                    Some(NoContextStep::FeatureExtAck(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    #[cfg(feature = "tds7.4")]
                    Some(NoContextStep::SessionState(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    Some(NoContextStep::Sspi(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    Some(NoContextStep::ServerError(span, next)) => {
                        head += span.bytes.len();
                        errors.push(span.own());
                        decoder = next;
                    }
                    Some(NoContextStep::EnvChange(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    Some(NoContextStep::Info(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    Some(NoContextStep::LoginAck(span, next)) => {
                        head += span.bytes.len();
                        decoder = next;
                    }
                    Some(NoContextStep::Done(span, _)) => {
                        let is_final = span.is_final();
                        if is_final {
                            done_token = Some(span.own());
                            break 'outer;
                        }
                        results.push(QueryResult {
                            done_token: span.own(),
                            errors: core::mem::take(&mut errors),
                            return_status: return_status.take(),
                        });
                        continue 'outer;
                    }
                    Some(NoContextStep::ReturnStatus(span, next)) => {
                        head += span.bytes.len();
                        buf.head = head;
                        return_status = Some(span.own());
                        decoder = next;
                    }
                    Some(NoContextStep::ReturnValue(span, next)) => {
                        head += span.byte_len();
                        decoder = next;
                    }
                    Some(NoContextStep::ContextRequired(ctx)) => {
                        let col_metadata_span = ctx.into_col_metadata();
                        head += 1 + col_metadata_span.bytes.len();
                        buf.head = head;
                        let col_metadata = col_metadata_span.own();
                        on_col_metadata(&col_metadata);
                        col_metadata_owned = Some(col_metadata);
                        continue 'outer; // re-enter drain
                    }
                    Some(NoContextStep::Error(_)) => {
                        buf.head = head;
                        break 'outer;
                    }
                    None => {
                        buf.head = head;
                        if buf.eof { break 'outer; }
                        buf.compact();
                        buf.fill(&mut self.stream).await?;
                        continue 'outer;
                    }
                }
            }
        }

        self.notify(Event::BytesReceived {
            heading: "QueryResponse",
            len: buf.tail,
        });

        let done_token = done_token.ok_or_else(|| {
        let peek = (buf.head < buf.tail)
            .then(|| unsafe { *buf.bytes[buf.head].as_ptr() });
            // let end = (buf.head + 32).min(buf.tail);
            let hexdump = {                                                                                                  
            let end = (buf.head + 32).min(buf.tail);                                                                   
            let mut bytes = [0u8; 32];                                                                                   
            let length = end - buf.head;
                unsafe {                                                                                                     
                    core::ptr::copy_nonoverlapping(                                                                        
                        buf.bytes[buf.head].as_ptr(),                                                       
                        bytes.as_mut_ptr(),                       
                        length,                                                                                              
                    );                                                                                                     
                }                                                                                                            
                HexDump { bytes, length }
            };   
            SessionError::UnexpectedEndOfStream {
                head: buf.head,
                tail: buf.tail,
                eof: buf.eof,
                peek,
                hexdump,
            }
        })?;
        results.push(QueryResult {
            done_token,
            errors,
            return_status,
        });

        Ok(QueryResults { results })
    }

}