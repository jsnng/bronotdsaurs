use core::mem::MaybeUninit;

use crate::tds::decoder::stream::{NoContextStep, TokenDecoder};
use crate::tds::prelude::*;
use crate::tds::session::prelude::*;
use crate::tds::session::routing::parse_routing;

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

impl QueryResults {
    pub fn errors(&self) -> Vec<ErrorInfoToken> {
        self.results
            .iter()
            .flat_map(|r| r.errors.iter().cloned())
            .collect()
    }
}

#[derive(Debug)]
pub struct DecodeOutput {
    pub results: QueryResults,
    pub transaction_descriptor: Option<u64>,
    pub routing: Option<Routing>,
}

#[derive(Debug, Default)]
pub struct LoginResponse {
    pub errors: Vec<ErrorInfoToken>,
    pub infos: Vec<ErrorInfoToken>,
    pub login_ack: bool,
    pub sspi_challenge: Option<Vec<u8>>,
    pub routing: Option<Routing>,
    pub transaction_descriptor: u64,
    pub packet_size: Option<usize>,
}

impl LoginResponse {
    pub(in crate::tds::session) fn new(
        readable: &[u8],
    ) -> Result<LoginResponse, SessionError> {
        let mut res = LoginResponse::default();
        let mut transaction_descriptor: Option<u64> = None;

        let mut decoder = TokenDecoder::new(readable);
        loop {
            match decoder.advance() {
                Some(NoContextStep::EnvChange(x, next)) => {
                    if let Some(EnvChangeType::PacketSize) = x.ty() {
                        res.packet_size = Some(parse_packet_size(&x));
                    }
                    apply_env_change(&x, &mut transaction_descriptor, &mut res.routing)?;
                    decoder = next;
                }
                Some(NoContextStep::ServerError(x, next)) => {
                    res.errors.push(x.own());
                    decoder = next;
                }
                Some(NoContextStep::Info(x, next)) => {
                    res.infos.push(x.own());
                    decoder = next;
                }
                Some(NoContextStep::LoginAck(_, next)) => {
                    res.login_ack = true;
                    decoder = next;
                }
                #[cfg(feature = "tds7.4")]
                Some(NoContextStep::FeatureExtAck(_, next)) => decoder = next,
                #[cfg(feature = "tds7.4")]
                Some(NoContextStep::SessionState(_, next)) => decoder = next,
                Some(NoContextStep::Sspi(x, next)) => {
                    res.sspi_challenge = Some(x.sspi_buffer().to_vec());
                    decoder = next;
                }
                Some(NoContextStep::Done(_, _)) | Some(NoContextStep::Error(_)) | None => break,
                _ => break,
            }
        }

        res.transaction_descriptor = transaction_descriptor.unwrap_or(0);
        Ok(res)
    }

}

#[inline]
fn apply_env_change(
    env_change_span: &crate::tds::prelude::EnvChangeSpan<'_>,
    transaction_descriptor: &mut Option<u64>,
    routing: &mut Option<Routing>,
) -> Result<(), SessionError> {
    match env_change_span.ty() {
        #[cfg(feature = "tds7.2")]
        Some(EnvChangeType::BeginTransaction)
        | Some(EnvChangeType::EnlistDTCTransaction) => {
            if env_change_span.bytes.len() >= 13 {
                *transaction_descriptor =
                    Some(u64::from_le_bytes(env_change_span.bytes[5..13].try_into().unwrap()));
            }
        }
        #[cfg(feature = "tds7.2")]
        Some(EnvChangeType::CommitTransaction)
        | Some(EnvChangeType::RollbackTransaction)
        | Some(EnvChangeType::DefectTransaction) => {
            *transaction_descriptor = Some(0);
        }
        #[cfg(feature = "tds7.4")]
        Some(EnvChangeType::SendRoutingInformation)
        | Some(EnvChangeType::SendEnhancedRoutingInformation) => {
            *routing = Some(parse_routing(env_change_span)?);
        }
        _ => {}
    }
    Ok(())
}

fn parse_packet_size(bytes: &crate::tds::prelude::EnvChangeSpan<'_>) -> usize {
    let env_value_data = bytes.env_value_data();
    let chars = env_value_data.first().copied().unwrap_or(0) as usize;
    let total = 1 + chars.saturating_mul(2);
    env_value_data.get(1..total)
        .and_then(|s| {
            s.chunks_exact(2).map(|c| c[0]).try_fold(0usize, |acc, b| {
                if b.is_ascii_digit() {
                    acc.checked_mul(10).and_then(|v| v.checked_add((b - b'0') as usize))
                } else {
                    None
                }
            })
        })
        .filter(|n| (512..=32_768).contains(n))
        .unwrap_or(4096)
}

pub(in crate::tds::session) struct StreamingBuffer {
    bytes: [MaybeUninit<u8>; 2 * MAX_TDS_PACKET_BYTES],
    head: usize,
    tail: usize,
    eof: bool,
}

impl StreamingBuffer {
    #[inline]
    pub(in crate::tds::session) fn new() -> Self {
        Self {
            bytes: [const { MaybeUninit::uninit() }; 2 * MAX_TDS_PACKET_BYTES],
            head: 0,
            tail: 0,
            eof: false,
        }
    }

    #[inline]
    fn compact(&mut self) {
        if self.head > 0 {
            let remaining = self.tail - self.head;
            self.bytes.copy_within(self.head..self.tail, 0);
            self.head = 0;
            self.tail = remaining;
        }
    }

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
            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                    self.bytes[self.tail + reading..self.tail + payload_length].as_mut_ptr()
                        as *mut u8,
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

impl<S, T: AsyncTransport, O: Observer<Event>> Session<S, T, O> {
    #[inline]
    pub(in crate::tds::session) async fn decode_token_stream<M, F>(
        &mut self,
        mut on_col_metadata: M,
        mut on_row: F,
    ) -> Result<DecodeOutput, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
    let stream = &mut self.stream;
    let mut buf = StreamingBuffer::new();
    let mut results: Vec<QueryResult> = Vec::new();
    let mut errors: Vec<ErrorInfoToken> = Vec::with_capacity(4);
    let mut col_metadata_owned: Option<ColMetaDataOwned> = None;
    let mut return_status: Option<ReturnStatusToken> = None;
    let mut done_token: Option<DoneToken> = None;
    let mut transaction_descriptor: Option<u64> = None;
    let mut routing: Option<Routing> = None;

    'outer: loop {
        if let Some(ref col_metadata) = col_metadata_owned {
            let col_metadata_span = col_metadata.borrow();
            let decoder = TokenDecoder::resume(
                unsafe {
                    let ptr = buf.bytes.as_ptr().add(buf.head) as *const u8;
                    core::slice::from_raw_parts(ptr, buf.tail - buf.head)
                },
                col_metadata_span,
            );
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
            let stalled_on = if buf.head < buf.tail {
                Some(unsafe { *buf.bytes[buf.head].as_ptr() })
            } else {
                None
            };
            match stalled_on {
                Some(0xd1) | Some(0xd2) => {
                    if buf.eof {
                        break 'outer;
                    }
                    buf.compact();
                    buf.fill(stream).await?;
                    continue 'outer;
                }
                Some(b) if b >= 0xfd => {
                    if buf.eof {
                        break 'outer;
                    }
                    buf.compact();
                    buf.fill(stream).await?;
                    continue 'outer;
                }
                None => {
                    if buf.eof {
                        break 'outer;
                    }
                    buf.compact();
                    buf.fill(stream).await?;
                    continue 'outer;
                }
                _ => {
                    col_metadata_owned = None;
                }
            }
        }

        let mut head = buf.head;
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
                    apply_env_change(&span, &mut transaction_descriptor, &mut routing)?;
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
                    continue 'outer;
                }
                Some(NoContextStep::Error(_)) => {
                    buf.head = head;
                    break 'outer;
                }
                None => {
                    buf.head = head;
                    if buf.eof {
                        break 'outer;
                    }
                    buf.compact();
                    buf.fill(stream).await?;
                    continue 'outer;
                }
            }
        }
    }

    let done_token = done_token.ok_or_else(|| {
        let peek = (buf.head < buf.tail).then(|| unsafe { *buf.bytes[buf.head].as_ptr() });
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

    Ok(DecodeOutput {
        results: QueryResults { results },
        transaction_descriptor,
        routing,
    })
    }
}