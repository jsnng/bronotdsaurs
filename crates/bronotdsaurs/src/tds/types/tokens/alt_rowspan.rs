use crate::tds::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum AggregateOperator {
    AOpStdDev = 0x30,
    AOpStdDevP = 0x31,
    AOpVar = 0x32,
    AOpVarP = 0x33,
    AOpCount = 0x4B,
    AOpSum = 0x4D,
    AOpAvg = 0x4F,
    AOpMin = 0x51,
    AOpMax = 0x52,
}

#[derive(Debug, Clone)]
pub struct AltRowToken {
    pub(crate) _id: u16,
    pub(crate) _compute_data: ComputeData,
}

#[derive(Debug, Clone)]
pub struct ComputeData {
    pub(crate) _op: u8,
    pub(crate) _operand: AggregateOperator,
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    pub(crate) _user_type: u16,
    #[cfg(feature = "tds7.2")]
    pub(crate) _user_type: u32,
    pub(crate) _flags: u8,
    pub(crate) _table_name: Vec<u16>,
    pub(crate) _col_name: Vec<u16>,
}

impl<'a> AltRowSpan<'a> {
    pub fn new() -> Result<Self, DecodeError> {
        todo!()
    }

    pub fn ty(&self) -> u8 { self.bytes[0] }
}
