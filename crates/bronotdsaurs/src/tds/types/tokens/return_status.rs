use crate::tds::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ReturnStatusToken {
    pub(crate) val: i32,
}

impl<'a> ReturnStatusSpan<'a> {
    pub const FIXED_SPAN_SIZE: usize = 5;
    pub fn new(bytes: &'a [u8]) ->  Result<Self, DecodeError> {
        if bytes.len() < Self::FIXED_SPAN_SIZE {
            return Err(DecodeError::InvalidData(format!("ReturnStatusSpan::new(): bytes.len() < {}. got {}", Self::FIXED_SPAN_SIZE, bytes.len())))
        }
        Ok(Self { bytes })
    }

    #[inline]
    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }

    #[inline]
    pub fn val(&self) -> i32 {
        r_i32_le(self.bytes, 1)
    }
}
