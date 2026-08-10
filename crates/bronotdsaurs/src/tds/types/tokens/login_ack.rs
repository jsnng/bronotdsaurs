use crate::tds::prelude::*;

impl<'a> LoginAckSpan<'a> {
    #[inline]
    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }

    #[inline]
    // length gives the token body size only.
    pub fn length(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }
    #[inline]
    pub fn interface(&self) -> u8 {
        self.bytes[3]
    }

    #[inline]
    pub fn tds_version(&self) -> [u8; 4] {
        [self.bytes[4], self.bytes[5], self.bytes[6],  self.bytes[7]]
    }

    #[inline]
    pub fn prog_name(&self) -> NVarCharSpan<'a> {
        NVarCharSpan::new(
            &self.bytes[self.ib_prog_name()..self.ib_prog_name() + self.cch_prog_name() * 2],
        )
    }

    pub const MIN_SPAN_SIZE: usize = 9;

    // Post:
    #[cfg_attr(kani, kani::ensures(|x|
        x.as_ref().map_or(true, |y|
            (y.length() as usize + 3 == bytes.len())
            && (y.bytes[0] == 0xad)
            && (Self::MIN_SPAN_SIZE + (y.bytes[8] as usize) * 2 <= y.bytes.len())
        )
    ))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::MIN_SPAN_SIZE {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("LoginAckSpan::new() bytes.len()={} < MIN_SPAN_SIZE={}", bytes.len(), Self::MIN_SPAN_SIZE))));
        }
        if bytes[0] != DataTokenType::LoginAck as u8 {
            return Err(kani_error_stubbed!(DecodeError::InvalidData(format!("LoginAckSpan::new() DataTokenType={} != DataTokenType::LoginAck {}", bytes[0], DataTokenType::LoginAck))));
        }
        let cch_prog_name = bytes[8] as usize;
        if Self::MIN_SPAN_SIZE + cch_prog_name*2 > bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("LoginAckSpan::new() ib_prog_name={}+cch_prog_name={} > bytes.len()={}", Self::MIN_SPAN_SIZE, cch_prog_name, bytes.len()))));
        }
        let login_ack = Self { bytes };
        // length gives the token body size only. bytes includes ty (u8) and length (u16).
        if login_ack.length() as usize + 3 != bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("LoginAckSpan::new() length+3={} != bytes.len()={}", login_ack.length() as usize + 3, bytes.len()))));
        }
        Ok(login_ack)
    }
    // index of prog_name start in self.bytes
    #[inline]
    pub const fn ib_prog_name(&self) -> usize {
        Self::MIN_SPAN_SIZE
    }
    // prog_name char count (unicode)
    #[inline]
    pub fn cch_prog_name(&self) -> usize {
        self.bytes[8] as usize
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct LoginAckToken {
    ty: u8,
    length: u16,
    interface: u8,
    tds_version: [u8; 4],
    prog_name: String,
}

impl LoginAckToken {
    #[inline]
    pub fn ty(&self) -> u8 {
        self.ty
    }

    #[inline]
    pub fn length(&self) -> u16 {
        self.length
    }

    #[inline]
    pub fn interface(&self) -> u8 {
        self.interface
    }

    #[inline]
    pub fn tds_version(&self) -> [u8; 4] {
        self.tds_version
    }

    #[inline]
    pub fn prog_name(&self) -> String {
        self.prog_name.clone()
    }
}


#[cfg(kani)]
#[kani::proof]
fn proof_login_ack_span_is_none() {
    let bytes: [u8; 8] = kani::any();
    assert!(LoginAckSpan::new(&bytes).is_err());
}

#[cfg(kani)]
#[kani::proof]
fn proof_prog_name() {
    let bytes: [u8; 32] = kani::any();
    if let Ok(span) = LoginAckSpan::new(&bytes) {     
        let _ = span.prog_name();
    }

    let bytes: [u8; 64] = kani::any();
    if let Ok(span) = LoginAckSpan::new(&bytes) {
        let _ = span.prog_name();
    }
}