use crate::tds::prelude::*;

span!(
    AltRowSpan,
    ColMetaDataItemSpan,
    DataClassificationSpan,
    DoneSpan,
    ColInfoSpan,
    ColInfoSpanIter,
    EnvChangeSpan,
    FeatureExtAckSpan,
    FedAuthInfoSpan,
    LoginAckSpan,
    #[cfg(feature = "tds7.3b")]
    NbcRowSpan,
    #[cfg(feature = "tds7.3b")]
    NbcRowColumnDataSpan,
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    OffsetSpan,
    OrderSpan,
    ReturnStatusSpan,
    #[cfg(feature = "tds7.3")]
    SessionStatusSpan,
    SspiSpan,
    TabNameSpan,
    TabNameSpanIter,
    NVarCharSpan,
    BVarBytesSpan,
    UsVarCharSpan,
    ColumnDataSpan,
);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromIntoFormat)]
pub enum DataTokenType {
    AltMetaData = 0x88,
    AltRow = 0xd3,
    ColMetaData = 0x81,
    ColInfo = 0xa5,
    #[cfg(feature = "tds7.4")]
    DataClassification = 0xa3,
    Done = 0xfd,
    DoneProc = 0xfe,
    DoneInProc = 0xff,
    EnvChange = 0xe3,
    Error = 0xaa,
    #[cfg(feature = "tds7.4")]
    FeatureExtAck = 0xae,
    #[cfg(feature = "tds7.4")]
    FedAuthInfo = 0xee,
    Info = 0xab,
    LoginAck = 0xad,
    #[cfg(feature = "tds7.3b")]
    NbcRow = 0xd2,
    #[cfg(all(feature = "tds7.2", not(feature = "tds7.3")))]
    Offset = 0x78,
    Order = 0xa9,
    ReturnStatus = 0x79,
    ReturnValue = 0xac,
    Row = 0xd1,
    #[cfg(feature = "tds7.4")]
    SessionState = 0xe4,
    Sspi = 0xed,
    TabName = 0xa4,
    TvpRow = 0x01,
}

impl DataTokenType {
    pub const LUT: [u8; 256] = {
        let mut x = [DataTokenType::UNKNOWN; 256];
        x[DataTokenType::ColMetaData as usize] = DataTokenType::COL_METADATA;
        x[DataTokenType::Done as usize] = DataTokenType::DONE;
        x[DataTokenType::DoneProc as usize] = DataTokenType::DONE;
        x[DataTokenType::DoneInProc as usize] = DataTokenType::DONE;
        x[DataTokenType::EnvChange as usize] = DataTokenType::ENV_CHANGE;
        x[DataTokenType::Info as usize] = DataTokenType::INFO;
        x[DataTokenType::LoginAck as usize] = DataTokenType::LOGIN_ACK;
        x[DataTokenType::Error as usize] = DataTokenType::ERROR;
        #[cfg(feature = "tds7.4")] {
            x[DataTokenType::FeatureExtAck as usize] = DataTokenType::FEATURE_EXT_ACK;
        }
        #[cfg(feature = "tds7.3b")] {
            x[DataTokenType::NbcRow as usize] = DataTokenType::NBC_ROW;
        }
        x[DataTokenType::Row as usize] = DataTokenType::ROW;
        x[DataTokenType::ReturnStatus as usize] = DataTokenType::RETURN_STATUS;
        x[DataTokenType::ReturnValue as usize] = DataTokenType::RETURN_VALUE;
        x[DataTokenType::ColInfo as usize] = DataTokenType::COL_INFO;
        x[DataTokenType::TabName as usize] = DataTokenType::TAB_NAME;
        #[cfg(feature = "tds7.4")] {
            x[DataTokenType::SessionState as usize] = DataTokenType::SESSION_STATE;
        }
        x[DataTokenType::Sspi as usize] = DataTokenType::SSPI;
        x[DataTokenType::Order as usize] = DataTokenType::ORDER;
        x
    };
    pub const UNKNOWN: u8 = 0;
    pub const COL_METADATA: u8 = 1;
    pub const DONE: u8 = 2;
    pub const ENV_CHANGE: u8 = 3;
    pub const INFO: u8 = 4;
    pub const LOGIN_ACK: u8 = 5;
    pub const ERROR: u8 = 6;
    pub const FEATURE_EXT_ACK: u8 = 7;
    pub const ROW: u8 = 8;
    #[cfg(feature = "tds7.3b")]
    pub const NBC_ROW: u8 = 9;
    pub const RETURN_STATUS: u8 = 10;
    pub const RETURN_VALUE: u8 = 11;
    pub const COL_INFO: u8 = 12;
    pub const TAB_NAME: u8 = 13;
    #[cfg(feature = "tds7.4")]
    pub const SESSION_STATE: u8 = 14;
    pub const SSPI: u8 = 15;
    pub const ORDER: u8 = 16;
}

impl<'a> BVarBytesSpan<'a> {
    #[inline(always)]
    // Post: Some(y) iff buf.len() == buf[0] + 1 AND &y.bytes == buf
    #[cfg_attr(kani, kani::ensures(|x: &Option<Self>|
        x.is_some() == (buf.len() == 1 + buf.first().copied().unwrap_or(0) as usize)
        && x.as_ref().map_or(true, |z| core::ptr::eq(z.bytes, buf))
    ))]
    pub fn new(buf: &'a [u8]) -> Option<Self> {
        if buf.is_empty() {
            return None;
        }
        if (buf[0] as usize) != buf.len() - 1 {
            return None;
        }

        Some(Self { bytes: buf })
    }
    #[inline(always)]
    pub fn length(&self) -> usize {
        self.bytes[0] as usize
    }
    #[inline(always)]
    pub fn bytes(&self) -> &'a [u8] {
        &self.bytes[1..]
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_varbytes_span_is_valid() {
    let bytes: [u8; 128] = kani::any();                                                                           
    let slice = kani::slice::any_slice_of_array(&bytes);                                                         
    if let Some(span) = BVarBytesSpan::new(slice) {
        assert_eq!(span.to_vec().len(), slice.len());
    }
}

impl PartialEq<str> for NVarCharSpan<'_> {
    fn eq(&self, other: &str) -> bool {
        let iter = self.bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]));
        iter.eq(other.encode_utf16())
    }
}

impl<'a> NVarCharSpan<'a> {
    #[inline(always)]
    // Pre: buf.len() % 2 == 0
    #[cfg_attr(kani, kani::requires(buf.len() % 2 == 0))]
    // Post: ptr::eq(y.bytes, buf) 
    #[cfg_attr(kani, kani::ensures(|x: &Self| core::ptr::eq(x.bytes, buf)))]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { bytes: buf }
    }
    #[inline(always)]
    pub fn characters(&self) -> usize {
        self.bytes.len() / 2
    }
}

impl<'a> core::fmt::Display for NVarCharSpan<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        let iter = self.bytes.chunks_exact(2).map(|x| u16::from_le_bytes([x[0], x[1]]));
        for char in core::char::decode_utf16(iter) {
            f.write_char(char.unwrap_or(core::char::REPLACEMENT_CHARACTER))?;
        }
        Ok(())
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_nvarchar_span_is_valid() {
    let bytes: [u8; 4] = kani::any();
    let _ = NVarCharSpan::new(&bytes);
}

proof_constructor_never_panics!(proof_col_info_never_panics, ColInfoSpan, 16);
proof_constructor_never_panics!(proof_order_never_panics, OrderSpan, 16);
proof_constructor_never_panics!(proof_tab_name_never_panics, TabNameSpan, 16);
#[cfg(feature = "tds7.3")]
proof_constructor_never_panics!(proof_session_status_never_panics, SessionStatusSpan, 16);