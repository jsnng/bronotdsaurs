use crate::tds::prelude::*;

#[cfg(kani)]
extern crate kani;

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum TokenType {
    ZeroLength = 0x10,
    FixedLength = 0x30,
    VariableLength = 0x20,
    VariableCount = 0x00,
}

impl TokenType {
    #[inline(always)]
    pub const fn mask() -> u8 {
        0x30
    }
}

#[derive(Debug, Clone)]
pub enum DataTokenSpan<'a> {
    AltMetaData(AltMetaDataSpan<'a>),
    AltRow(AltRowSpan<'a>),
    ColMetaData(ColMetaDataSpan<'a>),
    ColInfo(ColInfoSpan<'a>),
    #[cfg(feature = "tds7.4")]
    DataClassification(DataClassificationSpan<'a>),
    Done(DoneSpan<'a>),
    DoneProc(DoneSpan<'a>),
    DoneInProc(DoneSpan<'a>),
    EnvChange(EnvChangeSpan<'a>),
    Error(ErrorInfoSpan<'a>),
    #[cfg(feature = "tds7.4")]
    FeatureExtAck(FeatureExtAckSpan<'a>),
    #[cfg(feature = "tds7.4")]
    FedAuthInfo(FedAuthInfoSpan<'a>),
    Info(ErrorInfoSpan<'a>),
    LoginAck(LoginAckSpan<'a>),
    #[cfg(feature = "tds7.3b")]
    NbcRow(NbcRowSpan<'a>),
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    Offset(OffsetSpan<'a>),
    Order(OrderSpan<'a>),
    ReturnStatus(ReturnStatusSpan<'a>),
    ReturnValue(ReturnValueSpan<'a>),
    Row(RowSpan<'a>),
    #[cfg(feature = "tds7.4")]
    SessionState(SessionStatusSpan<'a>),
    Sspi(SspiSpan<'a>),
    TabName(TabNameSpan<'a>),
    TvpRow(RowSpan<'a>),
}

#[derive(Debug, Clone)]
pub enum DataToken {
    AltMetaData(AltMetaDataToken),
    AltRow(AltRowToken),
    ColMetaData(ColMetaDataToken),
    ColInfo(ColInfoToken),
    #[cfg(feature = "tds7.4")]
    DataClassification(DataClassificationToken),
    Done(DoneToken),
    DoneProc(DoneToken),
    DoneInProc(DoneToken),
    EnvChange(EnvChangeToken),
    Error(ErrorInfoToken),
    Info(ErrorInfoToken),
    LoginAck(LoginAckToken),
    #[cfg(feature = "tds7.3b")]
    NbcRow(NbcRowToken),
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    Offset(OffsetToken),
    Order(OrderToken),
    ReturnStatus(ReturnStatusToken),
    ReturnValue(ReturnValueToken),
    Row(RowToken),
    #[cfg(feature = "tds7.4")]
    SessionState(SessionStatusToken),
    TabName(TabNameToken),
    TvpRow(RowToken),
}

#[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
#[derive(Debug, Clone, Copy)]
pub struct OffsetToken {
    pub(crate) identifier: u16,
    pub(crate) offset_len: u16,
}

#[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
impl<'a> OffsetSpan<'a> {}

#[derive(Debug, Clone, Copy, DefaultDisplayFormat)]
pub enum TokenStreamType {
    BulkLoad,
    RPC,
    FederatedAuthenticationInformation,
    FeatureExtAck,
    LoginResponse,
    RowData,
    ReturnStatus,
    ReturnParameter,
    ResponseCompletion,
    SessionStatus,
    Error,
    AttentionAcknowledgement,
}
