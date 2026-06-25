use crate::tds::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ReturnValueSpan<'a> {
    bytes: &'a [u8],
    ib_param_name: usize,
    cch_param_name: usize,
    ib_status: usize,
    ib_user_type: usize,
    ib_flags: usize,
    ib_type_info: usize,
    ib_crypto_metadata: usize,
    ib_value: usize,
    cch_value: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Builder)]
#[builder(no_std)]
pub struct ReturnValueToken {
    ty: u8,
    param_ordinal: u16,
    param_name: String,
    status: u8,
    #[cfg(not(feature = "tds7.2"))]
    user_type: u16,
    #[cfg(feature = "tds7.2")]
    user_type: u32,
    flags: u16,
    type_info: Vec<u8>,
    crypto_metadata: Vec<u8>,
    value: Vec<u8>,
}

impl<'a> ReturnValueSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 4 {
            return Err(kani_error_stubbed!(DecodeError::InvalidData("ReturnValueSpan: insufficient bytes".to_string())));
        }
        let cch_param_name = bytes[3] as usize;
        let ib_param_name = 4;
        let ib_status = ib_param_name + cch_param_name * 2;
        let ib_user_type = ib_status + 1;
        #[cfg(feature = "tds7.2")]
        let ib_flags = ib_user_type + 4;
        #[cfg(not(feature = "tds7.2"))]
        let ib_flags = ib_user_type + 2;
        let ib_type_info = ib_flags + 2;

        if ib_type_info >= bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidData("ReturnValueSpan: insufficient bytes for type_info".to_string())));
        }
        let ty_id = bytes[ib_type_info];
        let ty_info: DtypeLUTEntry = DTYPE_LUT[ty_id as usize];
        let cch_type_info = ty_info.cch_type_info as usize;
        let stride = ty_info.stride;

        let ib_crypto_metadata = ib_type_info + 1 + cch_type_info;
        let ib_value = ib_crypto_metadata; // 0-size when not encrypted
        let cch_value = walk(bytes, ib_value, stride).unwrap_or(0);

        Ok(Self {
            bytes,
            ib_param_name,
            cch_param_name,
            ib_status,
            ib_user_type,
            ib_flags,
            ib_type_info,
            ib_crypto_metadata,
            ib_value,
            cch_value,
        })
    }

    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }
    pub fn param_ordinal(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }
    pub fn param_name(&self) -> NVarCharSpan<'a> {
        NVarCharSpan {
            bytes: &self.bytes[self.ib_param_name..self.ib_param_name + self.cch_param_name * 2],
        }
    }
    pub fn status(&self) -> u8 {
        self.bytes[self.ib_status]
    }
    #[cfg(not(feature = "tds7.2"))]
    pub fn user_type(&self) -> u16 {
        r_u16_le(self.bytes, self.ib_user_type)
    }
    #[cfg(feature = "tds7.2")]
    pub fn user_type(&self) -> u32 {
        r_u32_le(self.bytes, self.ib_user_type)
    }
    pub fn flags(&self) -> u16 {
        r_u16_le(self.bytes, self.ib_flags)
    }
    pub fn type_info(&self) -> &'a [u8] {
        &self.bytes[self.ib_type_info..self.ib_crypto_metadata]
    }
    pub fn crypto_metadata(&self) -> &'a [u8] {
        &self.bytes[self.ib_crypto_metadata..self.ib_value]
    }
    pub fn value(&self) -> &'a [u8] {
        &self.bytes[self.ib_value..self.ib_value + self.cch_value]
    }
    pub fn byte_len(&self) -> usize {
        self.ib_value + self.cch_value
    }
}
