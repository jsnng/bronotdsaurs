use crate::tds::prelude::*;

#[derive(Debug, Clone)]
pub struct SspiToken {
    pub(crate) ty: u8,
    pub(crate) sspi_buffer: Vec<u8>,
}

impl<'a> SspiSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 3 { return Err(DecodeError::InvalidData("".to_string()))}
        let length = r_u16_le(bytes, 1) as usize;
        if bytes.len() != 3 + length { return Err(DecodeError::InvalidData("".to_string()))}
        Ok(Self { bytes })
    }

    pub fn ty(&self) -> u8 { self.bytes[0] }
    pub fn length(&self) -> u16 { r_u16_le(self.bytes, 1)}
    pub fn sspi_buffer(&self) -> &'a [u8] { &self.bytes[3..] }
}
