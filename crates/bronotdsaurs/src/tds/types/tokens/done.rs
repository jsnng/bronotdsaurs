use crate::tds::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct DoneToken {
    pub(crate) ty: DataTokenType,
    pub(crate) status: u16,
    pub(crate) current_cmd: u16,
    #[cfg(not(feature = "tds7.2"))]
    pub(crate) done_row_count: u32,
    #[cfg(feature = "tds7.2")]
    pub(crate) done_row_count: u64,
}

impl DoneToken {
    pub fn as_bytes(&self) -> [u8; DoneSpan::FIXED_SPAN_SIZE] {
        let mut buf = [0u8; DoneSpan::FIXED_SPAN_SIZE];
        buf[0] = DataTokenType::Done as u8;
        buf[1..3].copy_from_slice(&self.status.to_le_bytes());
        buf[3..5].copy_from_slice(&self.current_cmd.to_le_bytes());
        #[cfg(not(feature = "tds7.2"))]
        buf[5..9].copy_from_slice(&self.done_row_count.to_le_bytes());
        #[cfg(feature = "tds7.2")]
        buf[5..13].copy_from_slice(&self.done_row_count.to_le_bytes());
        buf
    }
}

#[rustfmt::skip]
impl DoneToken {
    #[inline(always)]
    pub fn is_final(&self) -> bool { self.status & DoneStatus::More as u16 == 0 }
    #[inline(always)]
    pub fn is_more(&self) -> bool { self.status & DoneStatus::More as u16 != 0 }
    #[inline(always)]
    pub fn is_error(&self) -> bool { self.status & DoneStatus::Error as u16 != 0 }
    #[inline(always)]
    pub fn is_in_transaction(&self) -> bool { self.status & DoneStatus::InTransaction as u16 != 0 }
    #[inline(always)]
    pub fn is_count(&self) -> bool { self.status & DoneStatus::Count as u16 != 0 }
    #[cfg(not(feature = "tds7.2"))]
    #[inline(always)]
    pub fn done_row_count(&self) -> u32 { self.done_row_count }
    #[cfg(feature = "tds7.2")]
    #[inline(always)]
    pub fn done_row_count(&self) -> u64 { self.done_row_count }
    #[inline(always)]
    pub fn is_attention(&self) -> bool { self.status & DoneStatus::Attention as u16 != 0 }
    #[inline(always)]
    pub fn is_rpc_in_atch(&self) -> bool { self.status & DoneStatus::RPCInBatch as u16 != 0 }
    #[inline(always)]
    pub fn is_server_error(&self) -> bool { self.status & DoneStatus::ServerError as u16 != 0 }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum DoneStatus {
    Final = 0x00,
    More = 0x01,
    Error = 0x02,
    InTransaction = 0x04,
    Count = 0x10,
    Attention = 0x20,
    RPCInBatch = 0x80,
    ServerError = 0x100,
}

impl<'a> DoneSpan<'a> {
    #[inline(always)]
    pub fn is_final(&self) -> bool { self.status() & DoneStatus::More as u16 == 0 }
    #[cfg(not(feature = "tds7.2"))]
    pub const FIXED_SPAN_SIZE: usize = 9;

    #[cfg(feature = "tds7.2")]
    pub const FIXED_SPAN_SIZE: usize = 13;

    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::FIXED_SPAN_SIZE {
            return Err(DecodeError::InvalidLength(format!(
                "DoneSpan::new() bytes.len()={} < FIXED_SPAN_SIZE={}",
                bytes.len(),
                Self::FIXED_SPAN_SIZE
            )));
        }
        Ok(Self { bytes })
    }

    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn ty(&self) -> DataTokenType {
        DataTokenType::from_u8(self.bytes[0]).unwrap_or(DataTokenType::Done)
    }

    #[inline(always)]
    pub fn status(&self) -> u16 {
        let cursor: usize = 1;
        r_u16_le(self.bytes, cursor)
    }

    #[inline(always)]
    pub fn current_cmd(&self) -> u16 {
        let cursor: usize = 3;
        r_u16_le(self.bytes, cursor)
    }

    #[cfg(not(feature = "tds7.2"))]
    #[inline(always)]
    pub fn done_row_count(&self) -> u32 {
        let cursor: usize = 5;
        r_u32_le(self.bytes, cursor)
    }

    #[cfg(feature = "tds7.2")]
    #[inline(always)]
    pub fn done_row_count(&self) -> u64 {
        u64::from_le_bytes(self.bytes[5..13].try_into().unwrap())
    }
}

