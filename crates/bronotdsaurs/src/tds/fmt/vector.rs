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

impl<'a> core::fmt::Display for VectorSpan<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let sizeof = if self.ty == Self::FULL_PRECISION_FLOAT { 4 } else { 2 };
        f.write_str("[")?;
        for (i, c) in self.bytes.chunks_exact(sizeof).enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            let v = if self.ty == Self::FULL_PRECISION_FLOAT {
                f32::from_le_bytes([c[0], c[1], c[2], c[3]])
            } else {
                f16_to_f32(u16::from_le_bytes([c[0], c[1]]))
            };
            write!(f, "{v}")?;
        }
        f.write_str("]")
    }
}

fn f16_to_f32(h: u16) -> f32 {
    let h = h as u32;
    let sign = (h & 0x8000) << 16;
    let exp = (h & 0x7c00) >> 10;
    let mant = h & 0x03ff;
    let bits = if exp == 0 {
        if mant == 0 {
            sign
        } else {
            let mut e = 0i32;
            let mut m = mant;
            while m & 0x0400 == 0 {
                m <<= 1;
                e -= 1;
            }
            e += 1;
            m &= !0x0400;
            sign | (((e + 112) as u32) << 23) | (m << 13)
        }
    } else if exp == 0x1f {
        sign | 0x7f80_0000 | (mant << 13)
    } else {
        sign | ((exp + 112) << 23) | (mant << 13)
    };
    f32::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f16() {
        assert_eq!(f16_to_f32(0x3c00), 1.0);
        assert_eq!(f16_to_f32(0xc000), -2.0);
        assert_eq!(f16_to_f32(0x0000), 0.0);
    }

    #[test]
    fn display_full_precision() {
        let mut bytes = vec![0xa9, 0x01, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00];
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&(-2.0f32).to_le_bytes());
        let v = VectorSpan::new(&bytes).unwrap();
        assert_eq!(v.to_string(), "[1.5, -2]");
    }
}

#[cfg(kani)]
#[kani::proof_for_contract(VectorSpan::new)]
fn verify_vector_span_new() {
    let bytes: [u8; 16] = kani::any();
    let _ = VectorSpan::new(&bytes);
}