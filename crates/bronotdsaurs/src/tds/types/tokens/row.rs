use crate::tds::prelude::*;

span!(RowItemSpan, RowSpan);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RowToken {
    pub(crate) bytes: Vec<u8>,
    pub(crate) col_metadata: Option<ColMetaDataOwned>,
}

impl RowToken {
    /// Owned `RowSpan` with `ColMetaDataSpan`
    pub fn from_span(span: RowSpan, col_metadata: &ColMetaDataSpan) -> Self {
        Self {
            bytes: span.bytes.to_vec(),
            col_metadata: Some(col_metadata.own()),
        }
    }
}

type DecodeFn = for<'a> fn(&'a [u8]) -> Option<ValueRef<'a>>;

/// per-dtype LUT entry: metadata parsing info plus the value decoder.
/// yields everything needed for both ColMetaData parsing and row-value decoding.
#[derive(Debug, Clone, Copy)]
pub struct DtypeLUTEntry {
    pub cch_type_info: u8,
    /// Stride used by `walk()` to advance past a column value in a row:
    /// 0 = zero-length, non-zero fixed = byte count, 0x8n = variable with n-byte length prefix.
    pub stride: u8,
    /// Value decoder, or `None` for unsupported types.
    pub decoder: Option<DecodeFn>,
}

pub const DTYPE_LUT: [DtypeLUTEntry; 256] = {
    const fn e(cch_type_info: u8, stride: u8, decoder: Option<DecodeFn>) -> DtypeLUTEntry {
        DtypeLUTEntry { cch_type_info, stride, decoder }
    }
    const NONE: DtypeLUTEntry = DtypeLUTEntry { cch_type_info: 0, stride: 0, decoder: None };
    let mut x = [NONE; 256];
    // ZeroLength
    x[ZeroLengthDataType::Null as usize] = e(0, 0, Some(|_| Some(ValueRef::Null)));
    // Fixed (cch=0, stride=size)
    x[FixedLengthDataType::Int1 as usize] = e(0, 1, Some(|b| Some(ValueRef::Int1(b[0]))));
    x[FixedLengthDataType::Bit as usize] = e(0, 1, Some(|b| Some(ValueRef::Bit(b[0] != 0))));
    x[FixedLengthDataType::Int2 as usize] = e(0, 2,
        Some(|b| Some(ValueRef::Int2(r_i16_le(b, 0)))));
    // x[FixedLengthDataType::Decimal as usize] = e(0, 17, None); // Decimal (17-byte, unimplemented)
    x[FixedLengthDataType::Int4 as usize] = e(0, 4,
        Some(|b| Some(ValueRef::Int4(r_i32_le(b, 0)))));
    x[FixedLengthDataType::DateTim4 as usize] = e(0, 4,
        Some(|b| Some(ValueRef::DateTime4(b[..4].try_into().ok()?))));
    x[FixedLengthDataType::Flt4 as usize] = e(0, 4,
        Some(|b| Some(ValueRef::Float4(r_f32_le(b, 0)))));
    x[FixedLengthDataType::DateTime as usize] = e(0, 8,
        Some(|b| Some(ValueRef::DateTime(b[..8].try_into().ok()?))));
    x[FixedLengthDataType::Flt8 as usize] = e(0, 8,
        Some(|b| Some(ValueRef::Float8(f64::from_le_bytes(b[..8].try_into().ok()?)))));
    // x[FixedLengthDataType::Numeric as usize] = e(0, 17, None); // Numeric (17-byte, unimplemented)
    x[FixedLengthDataType::Money4 as usize] = e(0, 4,
        Some(|b| Some(ValueRef::Money4(b[0..4].try_into().ok()?))));
    x[FixedLengthDataType::Money as usize] = e(0, 8,
        Some(|b| Some(ValueRef::Money(b[0..8].try_into().ok()?))));
    x[FixedLengthDataType::Int8 as usize] = e(0, 8,
        Some(|b| Some(ValueRef::Int8(i64::from_le_bytes(b[..8].try_into().ok()?)))));
    // Variable BYTELEN (cch=1, stride=0x81) — b[0] is length, 0x00 = null
    x[VariableLengthDataType::Guid as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(ValueRef::Guid(b.get(1..17)?.try_into().ok()?))
    }));
    x[VariableLengthDataType::IntN as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(match b[0] {
            1 => ValueRef::Int1(b[1]),
            2 => ValueRef::Int2(r_i16_le(b, 1)),
            4 => ValueRef::Int4(r_i32_le(b, 1)),
            8 => ValueRef::Int8(i64::from_le_bytes(b[1..9].try_into().ok()?)),
            _ => return None,
        })
    }));
    #[cfg(feature = "tds7.3")]
    { 
        x[VariableLengthDataType::DateN as usize] = e(0, 0x81, None); 
        x[VariableLengthDataType::TimeN as usize] = e(1, 0x81, None); 
        x[VariableLengthDataType::DateTime2N as usize] = e(1, 0x81, None);
        x[VariableLengthDataType::DateTimeOffsetN as usize] = e(1, 0x81,  None); 
    }
    x[VariableLengthDataType::BitN as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(ValueRef::Bit(b[1] != 0))
        }));
    x[VariableLengthDataType::DecimalN as usize] = e(3, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(ValueRef::Decimal(&b[1..]))
    }));
    x[VariableLengthDataType::NumericN as usize] = e(3, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(ValueRef::Decimal(&b[1..]))
    }));
    x[VariableLengthDataType::FltN as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(match b[0] {
            4 => ValueRef::Float4(r_f32_le(b, 1)),
            8 => ValueRef::Float8(f64::from_le_bytes(b[1..9].try_into().ok()?)),
            _ => return None,
        })
    }));
    x[VariableLengthDataType::MoneyN as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(match b[0] {
            4 => ValueRef::Money4(b[1..5].try_into().ok()?),
            8 => ValueRef::Money(b[1..9].try_into().ok()?),
            _ => return None,
        })
    }));
    x[VariableLengthDataType::DateTimN as usize] = e(1, 0x81, Some(|b| {
        if b[0] == 0x00 { return Some(ValueRef::Null); }
        Some(match b[0] {
            4 => ValueRef::DateTime4(b[1..5].try_into().ok()?),
            8 => ValueRef::DateTime(b[1..9].try_into().ok()?),
            _ => return None,
        })
    }));
    // Variable USHORTLEN (cch=varies, stride=0x82) — b[0..2] is length, 0xffFF = null
    x[VariableLengthDataType::BigVarBinary as usize] = e(2, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarBinary(&b[2..]))
    }));
    x[VariableLengthDataType::BigVarChar as usize] = e(7, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarChar(&b[2..]))
    }));
    x[VariableLengthDataType::BigBinary as usize] = e(2, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarBinary(&b[2..]))
    }));
    x[VariableLengthDataType::BigChar as usize] = e(7, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarChar(&b[2..]))
    }));
    x[VariableLengthDataType::NVarChar as usize] = e(7, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::NVarChar(&b[2..]))
    }));
    x[VariableLengthDataType::NChar as usize] = e(7, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::NVarChar(&b[2..]))
    }));
    // Variable LONGLEN (cch=varies, stride=0x84) — b[0..4] is length, 0xffFFFFFF = null
    x[VariableLengthDataType::Image as usize] = e(4, 0x84, Some(|b| {
        if b[3] == 0xff && b[0] == 0xff && b[1] == 0xff && b[2] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarBinary(&b[4..]))
    }));
    x[VariableLengthDataType::Text as usize] = e(4, 0x84, Some(|b| {
        if b[3] == 0xff && b[0] == 0xff && b[1] == 0xff && b[2] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::VarChar(&b[4..]))
    }));
    #[cfg(feature = "tds7.2")]
    { x[VariableLengthDataType::SsVariant as usize] = e(4, 0x84, None); }
    x[VariableLengthDataType::NText as usize] = e(9, 0x84, Some(|b| {
        if b[3] == 0xff && b[0] == 0xff && b[1] == 0xff && b[2] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::NVarChar(&b[4..]))
    }));
    x[VariableLengthDataType::Vector as usize] = e(3, 0x82, Some(|b| {
        if b[1] == 0xff && b[0] == 0xff { return Some(ValueRef::Null); }
        Some(ValueRef::Vector(vector::VectorSpan::new(&b[2..]).ok()?))
    }));
    x
};

impl<'a> RowItemSpan<'a> {
    #[inline(always)]
    pub fn val_ref(&self, ty: u8) -> Option<ValueRef<'a>> {
        DTYPE_LUT[ty as usize].decoder?(self.bytes)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ValueRef<'a> {
    Null,
    Int1(u8),
    Int2(i16),
    Int4(i32),
    Int8(i64),
    Bit(bool),
    Float4(f32),
    Float8(f64),
    Money4([u8; 4]),
    Money([u8; 8]),
    DateTime4([u8; 4]),
    DateTime([u8; 8]),
    NVarChar(&'a [u8]), // UTF-16LE
    VarChar(&'a [u8]),
    VarBinary(&'a [u8]),
    Guid([u8; 16]),
    Decimal(&'a [u8]),
    Vector(vector::VectorSpan<'a>),
}

impl core::fmt::Display for ValueRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            ValueRef::Null => f.write_str("NULL"),
            ValueRef::Int1(x) => write!(f, "{x}"),
            ValueRef::Int2(x) => write!(f, "{x}"),
            ValueRef::Int4(x) => write!(f, "{x}"),
            ValueRef::Int8(x) => write!(f, "{x}"),
            ValueRef::Bit(x) => write!(f, "{x}"),
            ValueRef::Float4(x) => write!(f, "{x}"),
            ValueRef::Float8(x) => write!(f, "{x}"),
            ValueRef::Money4(b) => write!(f, "{}", money::SmallMoney::new(b)),
            ValueRef::Money(b) => write!(f, "{}", money::Money::new(b)),
            ValueRef::DateTime4(b) => write!(f, "{}", datetime::SmallDateTime::new(b)),
            ValueRef::DateTime(b) => write!(f, "{}", datetime::DateTime::new(b)),
            ValueRef::NVarChar(b) => write!(f, "{}", NVarCharSpan::new(b)),
            ValueRef::VarChar(b) => write!(f, "{}", String::from_utf8_lossy(b)),
            ValueRef::Guid(b) => write!(f, "{}", guid::Guid::new(b)),
            ValueRef::Vector(v) => write!(f, "{v}"),
            ValueRef::VarBinary(b) | ValueRef::Decimal(b) => {
                for byte in b {
                    write!(f, "{byte:02x}")?;
                }
                Ok(())
            }
        }
    }
}