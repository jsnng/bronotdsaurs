#![allow(dead_code)]
use crate::tds::prelude::*;

#[derive(Debug, Clone)]
pub struct AltMetaDataToken {
    pub(crate) count: u16,
    pub(crate) id: u16,
    pub(crate) by_cols: Vec<u16>,
    pub(crate) num_parts: u8,
    pub(crate) part_name: Vec<u16>,
    pub(crate) col_num: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
pub struct AltMetaDataSpan<'a> {
    pub bytes: &'a [u8],
    cch_by_cols: usize,
    ib_part_name: usize,
}

impl<'a> AltMetaDataSpan<'a> {
    pub const FIXED_SPAN_OFFSET: usize = 6;
    pub fn new(bytes: &'a [u8]) -> Self {
        let cch_by_cols = bytes[5] as usize;
        let ib_parts = AltMetaDataSpan::FIXED_SPAN_OFFSET + cch_by_cols * 2;
        let ib_part_name = ib_parts + 1;
        Self { bytes, cch_by_cols, ib_part_name }
    }

    fn ty(&self) -> u8 {
        self.bytes[0]
    }

    fn count(&self) -> u16 {
       r_u16_le(self.bytes, 1)
    }

    fn id(&self) -> u16 {
        r_u16_le(self.bytes, 3)
    }

    fn by_cols(&self) -> BVarBytesSpan<'a> {
        let count = self.bytes[5] as usize;
        BVarBytesSpan {
            bytes: &self.bytes[5..Self::FIXED_SPAN_OFFSET+count*2]
        }
    }

    fn num_parts(&self) -> u8 {
        self.bytes[self.ib_part_name - 1]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
enum AggregateOperator {
    StdDev = 0x30,
    StdDevP = 0x31,
    Var = 0x32,
    VarP = 0x33,
    Count = 0x4b,
    Sum = 0x4d,
    Avg = 0x4f,
    Min = 0x51,
    Max = 0x52,
}

struct ComputeData {
    op: AggregateOperator,
    operand: u16,
    #[cfg(feature = "tds7.2")]
    user_type: u32,
    #[cfg(not(feature = "tds7.2"))]
    user_type: u16,
    flags: u8,

}

struct ComputeDataFlag {
    f_nullable: bool,
    f_case_sen: bool,
    us_updateable: u8,
    f_indentity: bool,
    #[cfg(feature = "tds7.2")]
    f_compute: bool,
    #[cfg(feature = "tds7.2")]
    f_fixed_len_clr_type: bool,
}

impl ComputeDataFlag {
    
}