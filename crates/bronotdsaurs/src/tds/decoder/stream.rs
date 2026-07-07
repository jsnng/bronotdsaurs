//! Typestate iterator for TDS token-based packet data streams.
//!
//! For the token-based packet data streams, [`RowSpan`] and [`NbcRowSpan`] carry no column
//! boundary information - a preceding [`ColMetaDataSpan`] token provides this information.
//! Enforcement at compile-time of this requirement is performed by the typestate pattern:
//! i.e., Row/NbcRow tokens are only accessible from [`TokenDecoder<ContextRequired`], which
//! itself is only accessible if a `COL_METADATA` token has been observed.
//!
//! ## State Transitions
//! ```text
//! TokenDecoder<NoContext>
//!        |
//!        | (COL_METADATA)
//!        |
//!        ▼
//! TokenDecoder<ContextRequired>
//!        |
//!        | (DONE)
//!        |
//!        ▼
//! TokenDecoder<NoContext>
//! ```
use crate::tds::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct TokenDecoder<'a, S> {
    buf: &'a [u8],
    state: S,
}

#[derive(Debug, Clone, Copy)]
pub struct NoContext;

#[derive(Debug, Clone)]
pub struct ContextRequired<'a> {
    col_metadata: ColMetaDataSpan<'a>,
}

pub trait Drainable {
    const TOKEN: u8;
    fn steps(buf: &[u8], strides: &[u8]) -> Option<usize>;
}

pub struct Row;
pub struct NbcRow;

impl Drainable for Row {
    const TOKEN: u8 = 0xd1;
    #[inline(always)]
    fn steps(buf: &[u8], strides: &[u8]) -> Option<usize> {
        let mut cursor = 1usize;
        for &stride in strides {
            match walk(buf, cursor, stride) {
                Some(w) => cursor += w,
                None => return None,
            }
        }
        if cursor > buf.len() { return None; }
        Some(cursor)
    }
}

impl Drainable for NbcRow {
    const TOKEN: u8 = 0xd2;
    #[inline(always)]
    fn steps(buf: &[u8], strides: &[u8]) -> Option<usize> {
        let bitmap = strides.len().div_ceil(8);
        let mut cursor = 1 + bitmap;
        if cursor > buf.len() { return None }
        for (i, &stride) in strides.iter().enumerate() {
            let is_null = buf[1 + i / 8] >> (i % 8) & 1 == 1;
            if !is_null {
                match walk(buf, cursor, stride) {
                    Some(w) => cursor += w,
                    None => return None,
                }
            }
        }
        if cursor > buf.len() { return None; }
        Some(cursor)
    }
}

pub enum NoContextStep<'a> {
    EnvChange(EnvChangeSpan<'a>, TokenDecoder<'a, NoContext>),
    Info(ErrorInfoSpan<'a>, TokenDecoder<'a, NoContext>),
    LoginAck(LoginAckSpan<'a>, TokenDecoder<'a, NoContext>),
    #[cfg(feature = "tds7.4")]
    FeatureExtAck(FeatureExtAckSpan<'a>, TokenDecoder<'a, NoContext>),
    ContextRequired(TokenDecoder<'a, ContextRequired<'a>>),
    Done(DoneSpan<'a>, TokenDecoder<'a, NoContext>),
    ServerError(ErrorInfoSpan<'a>, TokenDecoder<'a, NoContext>),
    Error(DecodeError),
    ReturnStatus(ReturnStatusSpan<'a>, TokenDecoder<'a, NoContext>),
    ReturnValue(ReturnValueSpan<'a>, TokenDecoder<'a, NoContext>),
    #[cfg(feature = "tds7.4")]
    SessionState(SessionStatusSpan<'a>, TokenDecoder<'a, NoContext>),
    Sspi(SspiSpan<'a>, TokenDecoder<'a, NoContext>),
    // ColInfo(ColInfoSpan<'a>, TokenDecoder<'a, NoContext>),
}

impl core::fmt::Debug for NoContextStep<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EnvChange(..) => write!(f, "EnvChange"),
            Self::Info(..) => write!(f, "Info"),
            Self::LoginAck(..) => write!(f, "LoginAck"),
            #[cfg(feature = "tds7.4")]
            Self::FeatureExtAck(..) => write!(f, "FeatureExtAck"),
            Self::ContextRequired(..) => write!(f, "ContextRequired"),
            Self::Done(..) => write!(f, "Done"),
            Self::ServerError(..) => write!(f, "ServerError"),
            Self::Error(e) => write!(f, "Error({:?})", e),
            Self::ReturnStatus(..) => write!(f, "ReturnStatus"),
            Self::ReturnValue(..) => write!(f, "ReturnValue"),
            #[cfg(feature = "tds7.4")]
            Self::SessionState(..) => write!(f, "SessionState"),
            Self::Sspi(..) => write!(f, "Sspi"),
            // Self::ColInfo(..) => write!(f, "ColInfo"),
        }
    }
}

#[derive(Debug)]
pub enum ContextRequiredStep<'a> {
    Row(RowSpan<'a>, TokenDecoder<'a, ContextRequired<'a>>),
    #[cfg(feature = "tds7.3b")]
    NbcRow(NbcRowSpan<'a>, TokenDecoder<'a, ContextRequired<'a>>),
    Done(DoneSpan<'a>, TokenDecoder<'a, NoContext>),
    Error(DecodeError),
    DoneInProc(DoneSpan<'a>, TokenDecoder<'a, NoContext>),
}


/// See: 2.2.7 Packet Data Token Stream Definition
/// Decoder for token-based packet data streams.
/// This is implemented as a custom typestate iterator.
impl<'a> TokenDecoder<'a, NoContext> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            state: NoContext,
        }
    }

    /// General-purpose one-token-at-a-time stepper. "catch-all" function for stream token parsing.
    #[inline(always)]
    pub fn advance(self) -> Option<NoContextStep<'a>> {
        let mut buf = self.buf;
        loop {
            if buf.is_empty() {
                return None;
            }
            let ty = DataTokenType::LUT[buf[0] as usize];
            match ty {
                DataTokenType::COL_METADATA => {
                    if let Ok(col_metadata) = ColMetaDataSpan::new(&buf[1..]) {
                        let length = 1 + col_metadata.bytes.len();
                        return Some(NoContextStep::ContextRequired(TokenDecoder {
                            buf: &buf[length..],
                            state: ContextRequired { col_metadata },
                        }));
                    } else {
                        return Some(NoContextStep::Error(DecodeError::InvalidColMetaData(
                            "Failed to parse COL_METADATA".to_string(),
                        )));
                    }
                }
                // DataTokenType::ORDER => {

                // },
                DataTokenType::DONE => {
                    let cursor = DoneSpan::FIXED_SPAN_SIZE;
                    if cursor > buf.len() { return None; }
                    return Some(NoContextStep::Done(
                        DoneSpan {
                            bytes: &buf[..cursor],
                        },
                        TokenDecoder {
                            buf: &buf[cursor..],
                            state: NoContext,
                        },
                    ));
                }
                DataTokenType::RETURN_STATUS => {
                    let cursor = ReturnStatusSpan::FIXED_SPAN_SIZE;
                    if cursor > buf.len() { return None; }
                    return Some(NoContextStep::ReturnStatus(
                        ReturnStatusSpan { bytes: &buf[..cursor] },
                        TokenDecoder {
                            buf: &buf[cursor..],
                            state: NoContext,
                        },
                    ));
                }
                DataTokenType::RETURN_VALUE => {
                    return match ReturnValueSpan::new(buf) {
                        Ok(span) => {
                            let cursor = span.byte_len();
                            if cursor > buf.len() { return None; }
                            Some(NoContextStep::ReturnValue(
                                span,
                                TokenDecoder {
                                    buf: &buf[cursor..],
                                    state: NoContext,
                                },
                            ))
                        }
                        Err(e) => Some(NoContextStep::Error(e)),
                    };
                }
                #[cfg(feature = "tds7.4")]
                DataTokenType::SESSION_STATE => {
                    // SESSIONSTATE carries a u32 length, unlike the u16 tokens below.
                    if buf.len() < 5 { return None; }
                    let cursor = 5 + r_u32_le(buf, 1) as usize;
                    if cursor > buf.len() { return None; }
                    return match SessionStatusSpan::new(&buf[..cursor]) {
                        Ok(span) => Some(NoContextStep::SessionState(
                            span,
                            TokenDecoder {
                                buf: &buf[cursor..],
                                state: NoContext,
                            },
                        )),
                        Err(e) => Some(NoContextStep::Error(e)),
                    };
                }
                _ => {
                    // All remaining NoContext tokens are VariableLength (u16 prefix)
                    let cursor = var_len_token_size(buf)?;
                    let span = &buf[..cursor];
                    let next_buf = &buf[cursor..];
                    let next = TokenDecoder {
                        buf: next_buf,
                        state: NoContext,
                    };
                    match ty {
                        DataTokenType::ENV_CHANGE => return match EnvChangeSpan::new(span) {
                            Ok(s) => Some(NoContextStep::EnvChange(s, next)),
                            Err(e) => Some(NoContextStep::Error(e)),
                        },
                        DataTokenType::INFO => return match ErrorInfoSpan::new(span) {
                            Ok(s) => Some(NoContextStep::Info(s, next)),
                            Err(e) => Some(NoContextStep::Error(e)),
                        },
                        DataTokenType::LOGIN_ACK => return match LoginAckSpan::new(span) {
                            Ok(s) => Some(NoContextStep::LoginAck(s, next)),
                            Err(e) => Some(NoContextStep::Error(e)),
                        },
                        DataTokenType::ERROR => return match ErrorInfoSpan::new(span) {
                            Ok(s) => Some(NoContextStep::ServerError(s, next)),
                            Err(e) => Some(NoContextStep::Error(e)),
                        },
                        #[cfg(feature = "tds7.4")]
                        DataTokenType::FEATURE_EXT_ACK => {
                            let mut idx = 1;
                            loop {
                                if idx >= buf.len() { return None }
                                if buf[idx] == 0xff { idx += 1; break; }
                                if idx + 5 > buf.len() { return None }
                                idx += 5 + r_u32_le(buf, idx + 1) as usize
                            }
                            if idx > buf.len() { return None; }
                            return match FeatureExtAckSpan::new(&buf[..idx]) {
                                Ok(s) => Some(NoContextStep::FeatureExtAck(
                                    s,
                                    TokenDecoder { buf: &buf[idx..], state: NoContext },
                                )),
                                Err(e) => Some(NoContextStep::Error(e)),
                            };
                        },
                        DataTokenType::SSPI => return match SspiSpan::new(span) {
                            Ok(s) => Some(NoContextStep::Sspi(s, next)),
                            Err(e) => Some(NoContextStep::Error(e)),
                        },
                        _ => {
                            // Skip unhandled variable-length tokens (TabName, ColInfo, Order, etc.)
                            buf = next_buf;
                            continue;
                        }
                    }
                }
            }
        }
    }
}

impl<'a> TokenDecoder<'a, ContextRequired<'a>> {
    /// Consumes the decoder and returns the column metadata span it was carrying.
    ///
    /// The span still borrows the underlying stream buffer (`'a`), so it is only
    /// valid until the next buffer refill — call `.own()` on it to keep the
    /// metadata across refills, then use [`TokenDecoder::resume`] to continue
    /// row decoding with it.
    #[inline(always)]
    pub fn into_col_metadata(self) -> ColMetaDataSpan<'a> {
        self.state.col_metadata
    }

    /// Reconstructs a `ContextRequired` decoder positioned at `buf` using the given column
    /// metadata. Use this to resume row decoding after a buffer refill without re-scanning
    /// the column metadata token.
    #[inline(always)]
    pub fn resume(buf: &'a [u8], col_metadata: ColMetaDataSpan<'a>) -> Self {
        Self {
            buf,
            state: ContextRequired { col_metadata },
        }
    }
    /// Drains ROW/NBCROW tokens from the buffer. Returns `(done_span, bytes_consumed)`.
    /// `bytes_consumed` counts from the start of `self.buf` up to and including
    /// the DONE token if one was found, or up to the stall point otherwise.
    ///
    /// Handles both ROW (0xD1) and NBCROW (0xD2) tokens — SQL Server may freely
    /// mix them within a single result set (NBCROW for rows with many NULLs).
    #[inline]
    pub fn drain<F: FnMut(&'a [u8])>(self, mut f: F) -> (Option<DoneSpan<'a>>, usize) {
        let original_length: usize = self.buf.len();
        let mut buf = self.buf;
        let strides = self.state.col_metadata.strides_as_slice();
        loop {
            if buf.is_empty() { return (None, original_length - buf.len()); }
            let b = buf[0];
            let cursor = match b {
                Row::TOKEN => Row::steps(buf, strides),
                #[cfg(feature = "tds7.3b")]
                NbcRow::TOKEN => NbcRow::steps(buf, strides),
                _ => None,
            };
            if let Some(cursor) = cursor  {
                f(&buf[..cursor]);
                buf = &buf[cursor..];
                continue;
            }
            if b >= 0xfd { // DONE/DONEPROC/DONEINPROC
                if buf.len() < DoneSpan::FIXED_SPAN_SIZE { return (None, original_length - buf.len()); }
                let done = DoneSpan { bytes: &buf[..DoneSpan::FIXED_SPAN_SIZE] };
                if done.ty() == DataTokenType::DoneInProc as u8 {
                    buf = &buf[DoneSpan::FIXED_SPAN_SIZE..];
                    continue;
                }
                let consumed = original_length - buf.len() + DoneSpan::FIXED_SPAN_SIZE;
                return (Some(done), consumed);
            }
            return (None, original_length - buf.len());
        }
    }

    /// The ContextRequiredStep version of advance().
    #[inline(always)]
    pub fn advance(self) -> Option<ContextRequiredStep<'a>> {
        let mut buf = self.buf;
        loop {
            if buf.is_empty() {
                return None
            }
            let ty = DataTokenType::LUT[buf[0] as usize];
            match ty {
                DataTokenType::ROW => {
                    if let Some(cursor) = Row::steps(buf, self.state.col_metadata.strides_as_slice()) {
                        return Some(ContextRequiredStep::Row(
                            RowSpan {
                                bytes: &buf[..cursor],
                            },
                            TokenDecoder {
                                buf: &buf[cursor..],
                                state: self.state,
                            },
                        ));
                    } else {
                        return None
                    }
                }
                #[cfg(feature = "tds7.3b")]
                DataTokenType::NBC_ROW => {
                    if let Some(cursor) = NbcRow::steps(buf, self.state.col_metadata.strides_as_slice()) {
                        return Some(ContextRequiredStep::NbcRow(
                            NbcRowSpan {
                                bytes: &buf[..cursor],
                            },
                            TokenDecoder {
                                buf: &buf[cursor..],
                                state: self.state,
                            },
                        ));
                    } else {
                        return None
                    }
                }
                DataTokenType::DONE => {
                    let length = DoneSpan::FIXED_SPAN_SIZE;
                    if buf.len() < length {
                        return None;
                    }
                    let done_span = DoneSpan {
                        bytes: &buf[..length],
                    };
                    if done_span.ty() == DataTokenType::DoneInProc as u8 {
                        return Some(ContextRequiredStep::DoneInProc(
                            done_span,
                            TokenDecoder {
                                buf: &buf[length..],
                                state: NoContext,
                            },
                        ));
                    }
                    return Some(ContextRequiredStep::Done(
                        done_span,
                        TokenDecoder {
                            buf: &buf[length..],
                            state: NoContext,
                        },
                    ));
                }
                DataTokenType::RETURN_STATUS => {
                    if ReturnStatusSpan::FIXED_SPAN_SIZE > buf.len() {
                        return None;
                    }
                    buf = &buf[ReturnStatusSpan::FIXED_SPAN_SIZE..];
                }
                _ => {
                    // Skip variable-length tokens with u16 length prefix (Order, TabName, ColInfo, etc.)
                    let skip = var_len_token_size(buf)?;
                    buf = &buf[skip..];
                }

            }
        }
    }
}

#[inline]
fn var_len_token_size(buf: &[u8]) -> Option<usize> {
    if buf.len() < 3 {
        return None;
    }
    let size = 3 + r_u16_le(buf, 1) as usize;
    if size > buf.len() {
        return None;
    }
    Some(size)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tds::session::SessionBuffer;
    // 0000   04 01 01 cd 00 34 01 00 e3 37 00 01 14 58 00 63   .....4...7...X.c
    // 0010   00 68 00 61 00 6e 00 67 00 65 00 44 00 61 00 74   .h.a.n.g.e.D.a.t
    // 0020   00 61 00 57 00 61 00 72 00 65 00 68 00 6f 00 75   .a.W.a.r.e.h.o.u
    // 0030   00 73 00 65 00 06 6d 00 61 00 73 00 74 00 65 00   .s.e..m.a.s.t.e.
    // 0040   72 00 ab 8e 00 45 16 00 00 02 00 33 00 43 00 68   r....E.....3.C.h
    // 0050   00 61 00 6e 00 67 00 65 00 64 00 20 00 64 00 61   .a.n.g.e.d. .d.a
    // 0060   00 74 00 61 00 62 00 61 00 73 00 65 00 20 00 63   .t.a.b.a.s.e. .c
    // 0070   00 6f 00 6e 00 74 00 65 00 78 00 74 00 20 00 74   .o.n.t.e.x.t. .t
    // 0080   00 6f 00 20 00 27 00 58 00 63 00 68 00 61 00 6e   .o. .'.X.c.h.a.n
    // 0090   00 67 00 65 00 44 00 61 00 74 00 61 00 57 00 61   .g.e.D.a.t.a.W.a
    // 00a0   00 72 00 65 00 68 00 6f 00 75 00 73 00 65 00 27   .r.e.h.o.u.s.e.'
    // 00b0   00 2e 00 0d 54 00 43 00 41 00 55 00 2d 00 56 00   ....T.C.A.U.-.V.
    // 00c0   57 00 45 00 53 00 54 00 2d 00 44 00 57 00 00 01   W.E.S.T.-.D.W...
    // 00d0   00 00 00 e3 08 00 07 05 09 04 d0 00 34 00 e3 17   ............4...
    // 00e0   00 02 0a 75 00 73 00 5f 00 65 00 6e 00 67 00 6c   ...u.s._.e.n.g.l
    // 00f0   00 69 00 73 00 68 00 00 ab 76 00 47 16 00 00 01   .i.s.h...v.G....
    // 0100   00 27 00 43 00 68 00 61 00 6e 00 67 00 65 00 64   .'.C.h.a.n.g.e.d
    // 0110   00 20 00 6c 00 61 00 6e 00 67 00 75 00 61 00 67   . .l.a.n.g.u.a.g
    // 0120   00 65 00 20 00 73 00 65 00 74 00 74 00 69 00 6e   .e. .s.e.t.t.i.n
    // 0130   00 67 00 20 00 74 00 6f 00 20 00 75 00 73 00 5f   .g. .t.o. .u.s._
    // 0140   00 65 00 6e 00 67 00 6c 00 69 00 73 00 68 00 2e   .e.n.g.l.i.s.h..
    // 0150   00 0d 54 00 43 00 41 00 55 00 2d 00 56 00 57 00   ..T.C.A.U.-.V.W.
    // 0160   45 00 53 00 54 00 2d 00 44 00 57 00 00 01 00 00   E.S.T.-.D.W.....
    // 0170   00 ad 36 00 01 74 00 00 04 16 4d 00 69 00 63 00   ..6..t....M.i.c.
    // 0180   72 00 6f 00 73 00 6f 00 66 00 74 00 20 00 53 00   r.o.s.o.f.t. .S.
    // 0190   51 00 4c 00 20 00 53 00 65 00 72 00 76 00 65 00   Q.L. .S.e.r.v.e.
    // 01a0   72 00 00 00 00 00 0f 00 08 6b e3 13 00 04 04 34   r........k.....4
    // 01b0   00 30 00 39 00 36 00 04 34 00 30 00 39 00 36 00   .0.9.6..4.0.9.6.
    // 01c0   fd 00 00 00 00 00 00 00 00 00 00 00 00            .............

    // Tabular Data Stream
    // Type: Response (4)
    // Status: 0x01, End of message
    // Length: 461
    // Channel: 52
    // Packet Number: 1
    // Window: 0
    // Token - EnvChange
    //     Token length: 55
    //     Type: Database (1)
    //     New Value Length: 20
    //     New Value: XchangeDataWarehouse
    //     Old Value Length: 6
    //     Old Value: master
    // Token - Info
    //     Token length: 142
    //     SQL Error Number: 5701
    //     State: 2
    //     Class (Severity): 0
    //     Error message length: 51 characters
    //     Error message: Changed database context to 'XchangeDataWarehouse'.
    //     Server name length: 13 characters
    //     Server name: TCAU-VWEST-DW
    //     Process name length: 0 characters
    //     Line number: 1
    // Token - EnvChange
    //     Token length: 8
    //     Type: SQL Collation (7)
    //     New Value Length: 5
    //     Collate codepage: 1033
    //     Collate flags: 0x00d0
    //     Collate charset ID: 52
    //     Old Value Length: 0
    // Token - EnvChange
    //     Token length: 23
    //     Type: Language (2)
    //     New Value Length: 10
    //     New Value: us_english
    //     Old Value Length: 0
    // Token - Info
    //     Token length: 118
    //     SQL Error Number: 5703
    //     State: 1
    //     Class (Severity): 0
    //     Error message length: 39 characters
    //     Error message: Changed language setting to us_english.
    //     Server name length: 13 characters
    //     Server name: TCAU-VWEST-DW
    //     Process name length: 0 characters
    //     Line number: 1
    // Token - LoginAck
    //     Token length: 54
    //     Interface: 1
    //     TDS version: 0x74000004
    //     Server name: Microsoft SQL Server
    //     Server Version: 15.0.2155
    // Token - EnvChange
    //     Token length: 19
    //     Type: Packet size (4)
    //     New Value Length: 4
    //     New Value: 4096
    //     Old Value Length: 4
    //     Old Value: 4096
    // Token - Done
    //     .... ...0 .000 0000 = Status flags: 0x000
    //     Operation: 0x0000
    //     Row count: 0
    #[test]
    fn test_loginack() {
        let capture: [u8; 461] = [
            0x04, 0x01, 0x01, 0xcd, 0x00, 0x34, 0x01, 0x00, 0xe3, 0x37, 0x00, 0x01, 0x14, 0x58,
            0x00, 0x63, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x65, 0x00, 0x44,
            0x00, 0x61, 0x00, 0x74, 0x00, 0x61, 0x00, 0x57, 0x00, 0x61, 0x00, 0x72, 0x00, 0x65,
            0x00, 0x68, 0x00, 0x6f, 0x00, 0x75, 0x00, 0x73, 0x00, 0x65, 0x00, 0x06, 0x6d, 0x00,
            0x61, 0x00, 0x73, 0x00, 0x74, 0x00, 0x65, 0x00, 0x72, 0x00, 0xab, 0x8e, 0x00, 0x45,
            0x16, 0x00, 0x00, 0x02, 0x00, 0x33, 0x00, 0x43, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6e,
            0x00, 0x67, 0x00, 0x65, 0x00, 0x64, 0x00, 0x20, 0x00, 0x64, 0x00, 0x61, 0x00, 0x74,
            0x00, 0x61, 0x00, 0x62, 0x00, 0x61, 0x00, 0x73, 0x00, 0x65, 0x00, 0x20, 0x00, 0x63,
            0x00, 0x6f, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x65, 0x00, 0x78, 0x00, 0x74, 0x00, 0x20,
            0x00, 0x74, 0x00, 0x6f, 0x00, 0x20, 0x00, 0x27, 0x00, 0x58, 0x00, 0x63, 0x00, 0x68,
            0x00, 0x61, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x65, 0x00, 0x44, 0x00, 0x61, 0x00, 0x74,
            0x00, 0x61, 0x00, 0x57, 0x00, 0x61, 0x00, 0x72, 0x00, 0x65, 0x00, 0x68, 0x00, 0x6f,
            0x00, 0x75, 0x00, 0x73, 0x00, 0x65, 0x00, 0x27, 0x00, 0x2e, 0x00, 0x0d, 0x54, 0x00,
            0x43, 0x00, 0x41, 0x00, 0x55, 0x00, 0x2d, 0x00, 0x56, 0x00, 0x57, 0x00, 0x45, 0x00,
            0x53, 0x00, 0x54, 0x00, 0x2d, 0x00, 0x44, 0x00, 0x57, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0xe3, 0x08, 0x00, 0x07, 0x05, 0x09, 0x04, 0xd0, 0x00, 0x34, 0x00, 0xe3, 0x17,
            0x00, 0x02, 0x0a, 0x75, 0x00, 0x73, 0x00, 0x5f, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x67,
            0x00, 0x6c, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68, 0x00, 0x00, 0xab, 0x76, 0x00, 0x47,
            0x16, 0x00, 0x00, 0x01, 0x00, 0x27, 0x00, 0x43, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6e,
            0x00, 0x67, 0x00, 0x65, 0x00, 0x64, 0x00, 0x20, 0x00, 0x6c, 0x00, 0x61, 0x00, 0x6e,
            0x00, 0x67, 0x00, 0x75, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x65, 0x00, 0x74, 0x00, 0x74, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x20,
            0x00, 0x74, 0x00, 0x6f, 0x00, 0x20, 0x00, 0x75, 0x00, 0x73, 0x00, 0x5f, 0x00, 0x65,
            0x00, 0x6e, 0x00, 0x67, 0x00, 0x6c, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68, 0x00, 0x2e,
            0x00, 0x0d, 0x54, 0x00, 0x43, 0x00, 0x41, 0x00, 0x55, 0x00, 0x2d, 0x00, 0x56, 0x00,
            0x57, 0x00, 0x45, 0x00, 0x53, 0x00, 0x54, 0x00, 0x2d, 0x00, 0x44, 0x00, 0x57, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0xad, 0x36, 0x00, 0x01, 0x74, 0x00, 0x00, 0x04, 0x16,
            0x4d, 0x00, 0x69, 0x00, 0x63, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x6f, 0x00,
            0x66, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x51, 0x00, 0x4c, 0x00, 0x20, 0x00,
            0x53, 0x00, 0x65, 0x00, 0x72, 0x00, 0x76, 0x00, 0x65, 0x00, 0x72, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0f, 0x00, 0x08, 0x6b, 0xe3, 0x13, 0x00, 0x04, 0x04, 0x34, 0x00, 0x30,
            0x00, 0x39, 0x00, 0x36, 0x00, 0x04, 0x34, 0x00, 0x30, 0x00, 0x39, 0x00, 0x36, 0x00,
            0xfd, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut buf = SessionBuffer::default();
        buf.writeable()[..capture.len()].copy_from_slice(&capture);
        let _n = buf.tail(capture.len());
        // println!("{:?}", capture.len());

        let mut login_ack = None;
        let mut decoder = TokenDecoder::new(&buf.readable()[8..]);
        loop {
            match decoder.advance() {
                #[cfg(feature = "tds7.4")]
                Some(NoContextStep::FeatureExtAck(_, next)) => {
                    decoder = next;
                }
                Some(NoContextStep::LoginAck(span, _)) => {
                    login_ack = Some(span);
                    break;
                }
                Some(NoContextStep::EnvChange(_, next))
                | Some(NoContextStep::Info(_, next))
                | Some(NoContextStep::ServerError(_, next)) => decoder = next,
                Some(NoContextStep::Done(_, _)) | None => break,
                Some(NoContextStep::ContextRequired(_)) | Some(NoContextStep::Error(_)) => break,
                _ => unreachable!(),
            }
        }

        let val = login_ack.unwrap();

        // LoginAck token at capture[0x171]
        assert_eq!(val.ty(), DataTokenType::LoginAck as u8);
        assert_eq!(
            val.length(),
            u16::from_le_bytes([capture[0x172], capture[0x173]])
        );
        assert_eq!(val.interface(), capture[0x174]);
        assert_eq!(val.tds_version(), capture[0x175..0x179]);
        assert_eq!(val.prog_name(), *"Microsoft SQL Server\0\0");
    }

    #[test]
    fn truncated_row_returns_none() {
        let row = [0xd1, 0xde, 0xad, 0xbe, 0xef];
        let col_metadata = ColMetaDataSpan::from_parts(&[0x01, 0x00], collections::SmallBytes::from_slice(&[4]));
        let mut decoder: TokenDecoder<'_, ContextRequired<'_>> = TokenDecoder::resume(&row, col_metadata);
        let ctx = decoder.advance();
        if let Some(ContextRequiredStep::Row(span, next)) = ctx {
            assert_eq!(span.bytes, &[0xd1, 0xde, 0xad, 0xbe, 0xef]);
            decoder = next;
        } else {
            panic!("Expected Some(ContextRequiredStep::Row), got {:?}", ctx);
        }
        assert!(decoder.advance().is_none());

        let col_metadata = ColMetaDataSpan::from_parts(&[0x01, 0x00], collections::SmallBytes::from_slice(&[4]));
        let decoder: TokenDecoder<'_, ContextRequired<'_>> = TokenDecoder::resume(&row[..3], col_metadata);
        assert!(decoder.advance().is_none());
    }

    #[test]
    fn test_sql_batch_response_decode() {
        let capture: [u8; 51] = [
            0x04, 0x01, 0x00, 0x33, 0x00, 0x00, 0x01, 0x00, 0x81, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20,
            0x00, 0xA7, 0x03, 0x00, 0x09, 0x04, 0xD0, 0x00, 0x34, 0x03, 0x62, 0x00, 0x61, 0x00, 0x72, 0x00,
            0xD1, 0x03, 0x00, 0x66, 0x6F, 0x6F, 0xFD, 0x10, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];

        let mut buf = SessionBuffer::default();
        buf.writeable()[..capture.len()].copy_from_slice(&capture);
        let _n = buf.tail(capture.len());

        let decoder = TokenDecoder::new(&buf.readable()[8..]);

        let row_decoder = match decoder.advance() {
            Some(NoContextStep::ContextRequired(next)) => {
                let col_metadata = next.clone().into_col_metadata();
                assert_eq!(col_metadata.count(), 1);
                let column = (&col_metadata).into_iter().next().unwrap();
                assert_eq!(column.ty(), 0xA7);
                next
            }
            other => panic!("expected ContextRequired (COL_METADATA), got {:?}", other),
        };

        let done_decoder = match row_decoder.advance() {
            Some(ContextRequiredStep::Row(span, next)) => {
                assert_eq!(span.bytes, &[0xD1, 0x03, 0x00, 0x66, 0x6F, 0x6F]);
                next
            }
            other => panic!("expected Row, got {:?}", other),
        };

        match done_decoder.advance() {
            Some(ContextRequiredStep::Done(done, _)) => {
                assert_eq!(done.status() & DoneStatus::Count as u16, DoneStatus::Count as u16);
                assert_eq!(done.current_cmd(), 0x00C1);
                assert_eq!(done.done_row_count(), 1);
            }
            other => panic!("expected Done, got {:?}", other),
        }
    }
}