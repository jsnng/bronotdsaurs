use crate::tds::prelude::*;

#[derive(Debug, Clone, Builder)]
#[builder(no_std)]
pub struct ColInfoToken {
    ty: DataTokenType,
    length: u16,
    col_property: Vec<ColProperty>,
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std)]
pub struct ColProperty {
    col_num: u8,
    table_num: u8,
    status: u8,
    col_name: Option<String>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ColPropertyStatus {
    Expression = 0x4,
    Key = 0x8,
    Hidden = 0x10,
    DifferentName = 0x20,
}

impl<'a> ColInfoSpan<'a> {
    pub const FIXED_SPAN_OFFSET: usize = 3;
    // #[cfg_attr(kani, kani::ensures(
    //     |x: &Result<Self, DecodeError>|
    //     x.as_ref().map_or(true, |v| v.bytes.len() >= Self::FIXED_SPAN_SIZE)
    // ))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 3 { return Err(kani_error_stubbed!(DecodeError::InvalidData("ColInfoSpan self.bytes < 3".to_string()))) }
        Ok(Self { bytes })
    }
    pub fn ty(&self) -> u8 {
        self.bytes[0]
    }
    pub fn length(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }
}

impl<'a> IntoIterator for ColInfoSpan<'a> {
    type Item = ColInfoSpanItem<'a>;

    type IntoIter = ColInfoSpanIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ColInfoSpanIter {
            bytes: &self.bytes[Self::FIXED_SPAN_OFFSET..],
        }
    }
}

impl<'a> IntoIterator for &'a ColInfoSpan<'a> {
    type Item = ColInfoSpanItem<'a>;
    type IntoIter = ColInfoSpanIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ColInfoSpanIter {
            bytes: &self.bytes[ColInfoSpan::FIXED_SPAN_OFFSET..],
        }
    }
}

impl<'a> IntoIterator for &'a ColInfoToken {
    type Item = &'a ColProperty;
    type IntoIter = core::slice::Iter<'a, ColProperty>;

    fn into_iter(self) -> Self::IntoIter {
        self.col_property.iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColInfoSpanItem<'a> {
    pub(crate) col_num: usize,
    pub(crate) table_num: usize,
    pub(crate) status: u8,
    pub(crate) col_name: Option<NVarCharSpan<'a>>,
}

impl<'a> Iterator for ColInfoSpanIter<'a> {
    type Item = ColInfoSpanItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.len() < 3 { return None }
        let status = self.bytes[2];
        let col_num = self.bytes[0] as usize;
        let table_num = self.bytes[1] as usize;
        let mut col_name = None;
        let mut cursor = 3;
        if status & 0x20 != 0 {
            if self.bytes.len() < 5 { return None; }
            let cch_col_name = r_u16_le(self.bytes, 3);
            let ib_col_name = 5;
            cursor = ib_col_name + cch_col_name as usize *2;
            col_name = Some(
                NVarCharSpan {
                    bytes: self.bytes.get(ib_col_name..cursor)?
                }
            );
        }

        let item  = ColInfoSpanItem {
            col_num,
            table_num,
            status,
            col_name,
        };

        self.bytes = self.bytes.get(cursor..)?;
        Some(item)
    }
}
