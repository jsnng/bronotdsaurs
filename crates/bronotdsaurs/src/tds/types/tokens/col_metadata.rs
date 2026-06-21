#![allow(unused)]
use crate::tds::prelude::*;
use collections::SmallBytes;

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ColMetaDataToken {
    ty: DataTokenType,
    count: u16,
    #[cfg(feature = "tds8.0")]
    cek_table: Option<CekTable>,
    pub(crate) column_data: Vec<ColMetaDataItem>,
}

impl ColMetaDataToken {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.ty as u8);
        bytes.extend_from_slice(&self.count.to_le_bytes());
        for item in &self.column_data {
             bytes.extend_from_slice(&item.as_bytes());
        }
        bytes
    }
}

#[cfg(feature = "tds8.0")]
#[derive(Debug, Clone)]
pub struct CekTable {
    ek_value_count: u16,
    ek_info: Vec<EKInfo>,
}

#[cfg(feature = "tds8.0")]
#[derive(Debug, Clone)]
pub struct EKInfo {}

#[derive(Debug, Clone, Copy, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ColMetaDataFlags {
    f_nullable: bool,
    f_case_sen: bool,
    us_updateable: UsUpdateable,
    f_identity: bool,
    #[cfg(feature = "tds7.2")]
    f_computed: bool,
    #[cfg(not(feature = "tds7.3a"))]
    us_reserved_odbc: u8,
    #[cfg(feature = "tds7.3b")]
    f_sparse_column_set: bool,
    #[cfg(feature = "tds7.4")]
    f_encrypted: bool,
    #[cfg(feature = "tds7.4")]
    us_reserved3: bool,
    #[cfg(feature = "tds7.2")]
    f_fixed_len_clr_type: bool,
    us_reserved: u8,
    #[cfg(feature = "tds7.2")]
    f_hidden: bool,
    #[cfg(feature = "tds7.2")]
    f_key: bool,
    #[cfg(feature = "tds7.2")]
    f_nullable_unknown: bool,
}

impl ColMetaDataFlags {
    pub fn as_bytes(&self) -> [u8; 2] {
        let mut bytes: [u8; 2] = [0u8; 2];

        bytes[0] = self.f_nullable as u8
            | (self.f_case_sen as u8) << 1
            | (self.us_updateable as u8) << 2
            | (self.f_identity as u8) << 4;


        #[cfg(not(feature = "tds7.3a"))]
        { bytes[0] |= (self.us_reserved_odbc & 0x3) << 6; }

        #[cfg(feature = "tds7.3b")]
        { bytes[0] |= (self.f_sparse_column_set as u8) << 6; }

        #[cfg(feature = "tds7.4")] { 
        bytes[0] |= (self.f_encrypted as u8) << 7;
        bytes[1] |= self.us_reserved3 as u8; 
        }

        #[cfg(feature = "tds7.2")] { 
        bytes[0] |= (self.f_computed as u8) << 5;
        bytes[1] |= (self.f_fixed_len_clr_type as u8) << 1
        | (self.us_reserved & 0x1) << 2
        | (self.f_hidden as u8) << 3
        | (self.f_key as u8) << 4
        | (self.f_nullable_unknown as u8) << 5;
        }

        bytes
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UsUpdateable {
    ReadOnly = 0x00,
    ReadWrite = 0x01,
    Unknown = 0x02,
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ColMetaDataTableName {
    num_parts: u8,
    part_name: String,
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct CryptoMetaData {
    ordinal: u16,
    user_type: u32,
    base_type_info: TypeInfo,
    encryption_algorithm: EncryptionAlgorithm,
    norm_version: u8,
}

#[cfg(feature = "tds7.4")]
impl CryptoMetaData {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(7);
        bytes.extend_from_slice(&self.ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.user_type.to_le_bytes());
        bytes.extend_from_slice(&[self.encryption_algorithm as u8]);
        bytes.extend_from_slice(&self.norm_version.to_le_bytes());

        bytes
    }
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct DataClassification {}

#[cfg(feature = "tds7.4")]
#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum SensitivityRank {
    NotDefined = -1,
    None = 0,
    Low = 10,
    Medium = 20,
    High = 30,
    Critical = 40,
}

#[derive(Debug, Clone)]
pub struct ColMetaDataSpan<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) strides: SmallBytes<32>,
}

/// Owned version of `ColMetaDataSpan`
#[derive(Debug, Clone)]
pub struct ColMetaDataOwned {
    pub(crate) bytes: Vec<u8>,
    pub(crate) strides: Vec<u8>,
}

impl<'a> ColMetaDataSpan<'a> {
    pub const STRIDE_VARIABLE_MASK: u8 = 0x80;

    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 2 {
            return Err(DecodeError::InvalidColMetaData("col_metadata too small".to_string()));
        }
        let count = r_u16_le(bytes, 0) as usize;
        let mut ib = 2usize;
        let strides = SmallBytes::<32>::fill_with(count, |_| {
            let ib_type = ib + ColMetaDataItemSpan::FIXED_DATA_OFFSET + 2;
            let item = DTYPE_LUT[bytes[ib_type] as usize];
            let ib_cch_col_name = ib_type + 1 + item.cch_type_info as usize;
            let cch_col_name = bytes[ib_cch_col_name] as usize;
            ib = ib_cch_col_name + 1 + cch_col_name * 2;
            item.stride
        });
        Ok(Self {
            bytes: &bytes[..ib],
            strides,
        })
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        r_u16_le(self.bytes, 0) as usize
    }

    #[inline(always)]
    pub fn stride(&self, i: usize) -> u8 {
        self.strides[i]
    }

    #[inline(always)]
    pub fn strides_as_slice(&self) -> &[u8] {
        self.strides.as_slice()
    }

    #[inline(always)]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// Construct a `ColMetaDataSpan` from pre-parsed strides, skipping the parsing loop.
    /// `bytes` must be the original ColMetaData token bytes; `strides` must have been produced
    /// by a prior call to `ColMetaDataSpan::new()` on the same bytes.
    pub fn from_parts(bytes: &'a [u8], strides: SmallBytes<32>) -> Self {
        Self { bytes, strides }
    }

    pub fn own(&self) -> ColMetaDataOwned {
        ColMetaDataOwned {
            bytes: self.bytes.to_vec(),
            strides: self.strides.to_vec(),
        }
    }
}

impl ColMetaDataOwned {
    #[inline(always)]
    pub fn count(&self) -> usize {
        r_u16_le(&self.bytes, 0) as usize
    }

    #[inline(always)]
    pub fn strides_as_slice(&self) -> &[u8] {
        self.strides.as_slice()
    }

    #[inline(always)]
    pub fn borrow(&self) -> ColMetaDataSpan<'_> {
        ColMetaDataSpan {
            bytes: &self.bytes,
            strides: SmallBytes::from_slice(&self.strides),
        }
    }
}

impl<'a> IntoIterator for &'a ColMetaDataSpan<'a> {
    type Item = ColMetaDataItemSpan<'a>;
    type IntoIter = ColumnMetaDataSpanIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ColumnMetaDataSpanIter::new(&self.bytes[2..], self.count())
    }
}

impl<'a> IntoIterator for &'a ColMetaDataOwned {
    type Item = ColMetaDataItemSpan<'a>;
    type IntoIter = ColumnMetaDataSpanIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let count = r_u16_le(&self.bytes, 0) as usize;
        ColumnMetaDataSpanIter::new(&self.bytes[2..], count)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnMetaDataSpanIter<'a> {
    pub(crate) bytes: &'a [u8],
    remaining: usize,
}

impl<'a> ColumnMetaDataSpanIter<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8], remaining: usize) -> Self {
        Self { bytes, remaining }
    }
}

impl<'a> IntoIterator for ColMetaDataSpan<'a> {
    type Item = ColMetaDataItemSpan<'a>;
    type IntoIter = ColumnMetaDataSpanIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ColumnMetaDataSpanIter::new(&self.bytes[2..], self.count())
    }
}

impl<'a> Iterator for ColumnMetaDataSpanIter<'a> {
    type Item = ColMetaDataItemSpan<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let offset = ColMetaDataItemSpan::FIXED_DATA_OFFSET + 2;
        let cch_type_info = DTYPE_LUT[self.bytes[offset] as usize].cch_type_info as usize;
        let col_name_len_offset = offset + 1 + cch_type_info;
        let col_name_chars = self.bytes[col_name_len_offset] as usize;
        let size = col_name_len_offset + 1 + col_name_chars * 2;
        if size > self.bytes.len() {
            return None;
        }
        let item = ColMetaDataItemSpan {
            bytes: &self.bytes[..size],
        };
        self.bytes = &self.bytes[size..];
        self.remaining -= 1;
        Some(item)
    }
}

impl<'a> ColMetaDataItemSpan<'a> {
    #[cfg(not(feature = "tds7.2"))]
    pub const FIXED_DATA_OFFSET: usize = 2;
    #[cfg(feature = "tds7.2")]
    pub const FIXED_DATA_OFFSET: usize = 4;

    #[cfg(not(feature = "tds7.2"))]
    #[inline(always)]
    pub fn user_type(&self) -> u16 {
        r_u16_le(self.bytes, 0)
    }

    #[cfg(feature = "tds7.2")]
    #[inline(always)]
    pub fn user_type(&self) -> u32 {
        r_u32_le(self.bytes, 0)
    }

    #[inline(always)]
    pub fn type_info(&self) -> Option<TypeInfoSpan<'a>> {
        let dtype = DataType::try_from(self.ty()).ok()?;
        match dtype {
            DataType::ZeroLength(_) => None,
            _ => Some(TypeInfoSpan {
                dtype,
                bytes: &self.bytes
                    [self.ib_type_info() - 1..self.ib_type_info() + self.cch_type_info()],
            }),
        }
    }

    #[inline(always)]
    const fn ib_type_info(&self) -> usize {
        Self::FIXED_DATA_OFFSET + 3
    }

    #[inline(always)]
    pub fn ty(&self) -> u8 {
        self.bytes[self.ib_type_info() - 1]
    }

    #[inline(always)]
    fn cch_type_info(&self) -> usize {
        DTYPE_LUT[self.ty() as usize].cch_type_info as usize
    }

    #[inline(always)]
    pub fn col_name(&self) -> NVarCharSpan<'a> {
        let ib = self.ib_col_name();
        let cch = self.bytes[ib - 1] as usize;
        NVarCharSpan {
            bytes: &self.bytes[ib..ib + cch * 2],
        }
    }

    #[inline(always)]
    pub fn ib_col_name(&self) -> usize {
        self.ib_type_info() + self.cch_type_info() + 1
    }

    #[inline(always)]
    pub fn cch_col_name(&self) -> usize {
        self.bytes[self.ib_col_name() - 1] as usize
    }
}

/// Advance cursor past the value at `buf[offset..]` for a column with the given stride.
/// Returns `None` if `buf` does not contain enough bytes (field spans a packet boundary).
#[inline(always)]
pub fn walk(buf: &[u8], offset: usize, stride: u8) -> Option<usize> {
    if stride == 0 {
        Some(0)
    } else if stride & ColMetaDataSpan::STRIDE_VARIABLE_MASK == 0 {
        Some(stride as usize)
    } else {
        let prefix_length = (stride & !ColMetaDataSpan::STRIDE_VARIABLE_MASK) as usize;
        if offset + prefix_length > buf.len() {
            return None;
        }
        let data_length = match prefix_length {
            #[cfg(feature = "unsafe")]
            1 => (unsafe { *buf.get_unchecked(offset) }) as usize, 
            #[cfg(not(feature = "unsafe"))] 
            1 => buf[offset] as usize,
            2 => {
                let length = r_u16_le(buf, offset);
                if length == 0xffff { return Some(2) }
                length as usize
            }
            _ => {
                let length = r_u32_le(buf, offset);
                if length == 0xffffffff { return Some(4) }
                length as usize
            }
        };
        Some(prefix_length + data_length)
    }
}


#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option), build_fn(validate = "Self::validate"))]
pub struct ColMetaDataItem {
    #[cfg(not(feature = "tds7.2"))]
    user_type: u16,
    #[cfg(feature = "tds7.2")]
    user_type: u32,
    flags: ColMetaDataFlags,
    pub(crate) type_info: TypeInfo,
    #[cfg(feature = "legacy")]
    table_name: Option<ColMetaDataTableName>,
    #[cfg(feature = "tds7.4")]
    crypto_meta_data: Option<CryptoMetaData>,
    col_name: String,
}

impl ColMetaDataItemBuilder {
    fn validate(&self) -> Result<(), String> {
        #[cfg(feature = "tds7.4")] 
        if self.flags.is_some_and(|x| x.f_encrypted) && self.crypto_meta_data.is_none() {
            return Err("f_encrypted is set but self.crypto_meta_data is None".to_string())
        }
        Ok(())
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_col_metadata_span_short_input_is_err() {
    let bytes: [u8; 1] = kani::any();
    assert!(ColMetaDataSpan::new(&bytes).is_err());
}

/// Contract: `ColMetaDataSpan::new` must never panic on arbitrary wire input —
/// truncated or corrupt COL_METADATA must surface as `Err`, and any `Ok` span
/// must lie within the input it was parsed from.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(8)]
fn proof_col_metadata_span_new_never_panics() {
    let bytes: [u8; 16] = kani::any();
    let slice = kani::slice::any_slice_of_array(&bytes);
    if slice.len() >= 2 {
        // keep the wire-declared column count small so the parse loop unwinds;
        // truncation bugs reproduce at any count >= 1
        kani::assume(slice[0] <= 4 && slice[1] == 0);
    }
    if let Ok(span) = ColMetaDataSpan::new(slice) {
        assert!(span.byte_len() <= slice.len());
    }
}

impl ColMetaDataItem {
    /// # SAFETY:
    /// [`ColMetaDataItemBuilder::validate`] guarantees `crypto_meta_data` is Some when `f_encrypted` is set.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(5);
        #[cfg(not(feature = "tds7.2"))]
        buf.extend_from_slice(&u16::to_le_bytes(self.user_type));
        #[cfg(feature = "tds7.2")]
        buf.extend_from_slice(&u32::to_le_bytes(self.user_type));
        buf.extend_from_slice(&self.flags.as_bytes());
        buf.extend_from_slice(&self.type_info.as_bytes());
        #[cfg(feature = "tds7.4")] 
        if self.flags.f_encrypted {
            buf.extend_from_slice(&self.crypto_meta_data.as_ref().unwrap().as_bytes());
        }
        let utf16: Vec<u8> = self.col_name.encode_utf16()
        .flat_map(|x| x.to_le_bytes())
        .collect();
        buf.extend_from_slice(&[utf16.len() as u8/2]);
        buf.extend_from_slice(&utf16);
        buf
    }
}
