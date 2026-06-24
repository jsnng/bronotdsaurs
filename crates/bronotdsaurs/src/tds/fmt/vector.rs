use crate::tds::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct VectorSpan<'a> {
    ty: u8,
    count: u16,
    bytes: &'a [u8],
}

impl<'a> VectorSpan<'a> {
    const HEADER_SIZE: usize = 8;
    const MAX_SIZE: usize = 8000;
    const LAYOUT_FORMAT: u8 = 0xa9;
    const LAYOUT_VERSION: u8 = 0x01;
    const HALF_PRECISION_FLOAT: u8 = 0x00;
    const FULL_PRECISION_FLOAT: u8 = 0x01;

    // Post: 
    #[cfg_attr(kani, kani::ensures(|x: &Result<VectorSpan, DecodeError>|
        x.as_ref().map_or(true, |v|
            v.bytes.len() == v.count as usize
                * if v.ty == Self::FULL_PRECISION_FLOAT { 4 } else { 2 }
            && v.bytes.len() <= Self::MAX_SIZE - Self::HEADER_SIZE
        )
    ))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        let header = bytes
            .get(..Self::HEADER_SIZE)
            .ok_or(crate::kani_error_stubbed!(DecodeError::InvalidLength("".to_string())))?;
        if header[0] != Self::LAYOUT_FORMAT || header[1] != Self::LAYOUT_VERSION {
            return Err(crate::kani_error_stubbed!(DecodeError::InvalidData("".to_string())));
        }

        let count = r_u16_le(header, 2);
        let ty = header[4];
        let sizeof: usize = match ty {
            Self::HALF_PRECISION_FLOAT => 2,
            Self::FULL_PRECISION_FLOAT => 4,
            _ => return Err(crate::kani_error_stubbed!(DecodeError::InvalidField("".to_string()))),
        };

        let total = count as usize * sizeof;
        if total + Self::HEADER_SIZE > Self::MAX_SIZE {
            return Err(crate::kani_error_stubbed!(DecodeError::InvalidLength("".to_string())))
        }
        let payload = bytes
            .get(Self::HEADER_SIZE..Self::HEADER_SIZE + total)
            .ok_or(crate::kani_error_stubbed!(DecodeError::InvalidLength("".to_string())))?;

        Ok(Self {
            ty,
            count,
            bytes: payload,
        })
    }

    pub fn ty(&self) -> u8 { 
        self.ty
    }

    pub fn count(&self) -> u16 {
        self.count
    }

    pub fn payload(&self) -> &'a [u8] {
        self.bytes
    }
}

#[cfg(kani)]
#[kani::proof_for_contract(VectorSpan::new)]
fn verify_vector_span_new() {
    let bytes: [u8; 16] = kani::any();
    let _ = VectorSpan::new(&bytes);
}