use crate::tds::types::prelude::*;


#[cfg(feature = "tds7.3b")]
impl<'a> NbcRowSpan<'a> {}

#[cfg(feature = "tds7.3b")]
#[derive(Debug, Clone)]
pub struct NbcRowToken {
    pub(crate) null_bitmap: Vec<u8>,
    pub(crate) all_column_data: Vec<NbcRowColumnData>,
}

#[cfg(feature = "tds7.3b")]
#[derive(Debug, Clone)]
pub struct NbcRowColumnData {
    pub(crate) text_pointer: Option<Vec<u8>>,
    pub(crate) timestamp: Option<[u8; 8]>,
    pub(crate) data: String,
}

#[cfg(feature = "tds7.3b")]
impl<'a> NbcRowColumnDataSpan<'a> {}