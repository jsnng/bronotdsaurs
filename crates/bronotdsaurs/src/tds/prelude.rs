pub use alloc::format;
pub use core::mem::size_of;
pub use crate::tds::decoder::error::DecodeError;
pub use crate::tds::decoder::rows::RowSpanIter;
pub use crate::tds::encoder::error::EncodeError;
pub use crate::tds::encoder::traits::MessageEncoder;
pub use crate::tds::types::prelude::*;
pub use traits::{
    Decode,
    Encoder,
};
pub use crate::tds::fmt::prelude::*;

#[cfg(feature = "tds8.0")]
// TDS Version 8.0 LE
pub const TDS_80: u32 = 0x80000001;

#[cfg(feature = "tds7.4")]
// TDS Version 7.4 LE
pub const TDS_74: u32 = 0x74000004;

#[cfg(feature = "tds7.3")]
// TDS Version 7.3 LE
pub const TDS_73: u32 = 0x730B0003;

#[cfg(feature = "tds7.2")]
// TDS Version 7.2 LE
pub const TDS_72: u32 = 0x72090002;

#[cfg(feature = "tds7.1")]
// TDS Version 7.1 LE
pub const TDS_71: u32 = 0x71000001;

#[cfg(feature = "tds7.0")]
// TDS Version 7.0 LE
pub const TDS_70: u32 = 0x70000000;