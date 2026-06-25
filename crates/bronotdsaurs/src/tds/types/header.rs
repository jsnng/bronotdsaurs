use crate::tds::prelude::*;

#[cfg(kani)]
extern crate kani;

span!(
    QueryNotificationHeaderSpan,
    TraceActivityHeaderSpan,
    TransactionDescriptorHeaderSpan,
);
// Message Header Size
// Client = 8
// PreLogin = 18,
// Login = 16,
// LoginIntegratedAuthentication = 16+17,
// FederatedAuthenticationInformation = 8,
// SQLBatch = 1,
// BulkLoad = 7,
// RPC = 3,
// Attention = 6,
// TransactionManagerRequest = 15,
// Server -
// PreLoginResponse = 4,
// LoginResponse = 4,
// FederatedAuthenticationInformation = 4,
// RowData = 4,
// ReturnStatus = 4
// ReturnParameters = 4,
// ResponseCompletion = 4,
// SessionState = 4,
// Error = 4,
// Attention Ack = 4

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromIntoFormat)]
pub enum MessageStateStatus {
    Normal = 0x00,
    EndOfMessage = 0x01,
    Ignore = 0x02,
    ResetConnection = 0x08,
    ResetConnectionSkipTransaction = 0x10,
}

impl core::ops::BitAnd<MessageStateStatus> for u8 {
    type Output = u8;

    #[inline(always)]
    fn bitand(self, rhs: MessageStateStatus) -> Self::Output {
        self & (rhs as u8)
    }
}

/// 2.2.5.3 Packet Data Stream Headers - ALL_HEADERS Rule Definition
#[derive(Debug, Clone)]
pub enum DataStreamHeaderType {
    QueryNotification(QueryNotificationHeader),
    TransactionDescriptor(TransactionDescriptorHeader),
    #[cfg(feature = "tds7.4")]
    TraceActivity(TraceActivityHeader),
}

impl core::fmt::Display for DataStreamHeaderType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DataStreamHeaderType::QueryNotification(_) => write!(f, "QueryNotification"),
            DataStreamHeaderType::TransactionDescriptor(_) => write!(f, "TransactionDescriptor"),
            #[cfg(feature = "tds7.4")]
            DataStreamHeaderType::TraceActivity(_) => write!(f, "TraceActivity"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllHeaders(Vec<DataStreamHeaderType>);

impl AllHeaders {
    pub fn new(mut headers: Vec<DataStreamHeaderType>) -> Self {
        if !headers.iter().any(|x| matches!(x, DataStreamHeaderType::TransactionDescriptor(_))) {
            headers.push(DataStreamHeaderType::TransactionDescriptor(TransactionDescriptorHeader::auto_commit()));
        }
        Self(headers)
    }

    pub fn with_transaction(td: u64) -> Self {
        Self(alloc::vec![DataStreamHeaderType::TransactionDescriptor(
            TransactionDescriptorHeader::with_transaction_descriptor(td, 1),
        )])
    }

    pub fn has_transaction_descriptor(&self) -> bool {
        self.0.iter().any(|x| matches!(x, DataStreamHeaderType::TransactionDescriptor(_)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &DataStreamHeaderType> {
        self.0.iter()
    }
}

/// 2.2.5.3.1 Query Notifications Header
#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct QueryNotificationHeader {
    pub(crate) notify_id: String,      // ushort unicodestream
    pub(crate) ssb_deployment: String, // ushort unicodestream
    pub(crate) notify_timeout: u32,    //ulong
}

impl core::ops::BitAnd<QueryNotificationHeader> for u8 {
    type Output = u8;

    #[inline(always)]
    fn bitand(self, _rhs: QueryNotificationHeader) -> Self::Output {
        self & 0x01
    }
}

impl<'a> QueryNotificationHeaderSpan<'a> {
    
    pub fn notify_id(&self) -> NVarCharSpan<'a> {
        todo!()
    }
    pub fn ssb_deployment(&self) -> NVarCharSpan<'a> {
        todo!()
    }
    pub fn notify_timeout(&self) -> u32 {
        todo!()
    }
}

/// 2.2.5.3.2 Transaction Descriptor Header
#[derive(Debug, Clone, Copy, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct TransactionDescriptorHeader {
    pub(crate) outstanding_request_count: u32, // dword
    pub(crate) transaction_descriptor: u64,    // ulonglong
}

impl TransactionDescriptorHeader {
    /// If the connection is operating in AutoCommit mode.
    pub fn auto_commit() -> Self {
        Self {
            outstanding_request_count: 1,
            transaction_descriptor: 0,
        }
    }

    pub fn with_transaction_descriptor(td: u64, outstanding: u32) -> Self {
        Self {
            outstanding_request_count: outstanding,
            transaction_descriptor: td,
        }
    }
}

impl core::ops::BitAnd<TransactionDescriptorHeader> for u8 {
    type Output = u8;

    #[inline(always)]
    fn bitand(self, _rhs: TransactionDescriptorHeader) -> Self::Output {
        self & 0x02
    }
}

impl TransactionDescriptorHeader {
    pub const LENGTH: usize = 12;
}

impl<'a> TransactionDescriptorHeaderSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() != TransactionDescriptorHeader::LENGTH {
            return Err(DecodeError::InvalidData("".to_string()))
        }
        Ok(Self { bytes })
    }
    pub fn transaction_descriptor(&self) -> u64 {
        u64::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3],
            self.bytes[4], self.bytes[5], self.bytes[6], self.bytes[7]])
    }
    pub fn outstanding_request_count(&self) -> u32 {
        u32::from_le_bytes([self.bytes[8], self.bytes[9], self.bytes[10], self.bytes[11]])
    }
}

#[allow(unused)]
#[cfg(feature = "tds7.4")]
/// 2.2.5.3.3 Trace Activity Header
#[derive(Debug, Clone, Copy, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct TraceActivityHeader {
    pub(crate) guid_activity_id: [u8; 16], // 16bytes
    pub(crate) activity_sequence: u32,     // ulong
}

#[cfg(feature = "tds7.4")]
impl core::ops::BitAnd<TraceActivityHeader> for u8 {
    type Output = u8;

    #[inline(always)]
    fn bitand(self, _rhs: TraceActivityHeader) -> Self::Output {
        self & 0x03
    }
}

#[cfg(feature = "tds7.4")]
impl TraceActivityHeader {
    pub const LENGTH: usize = 20;
}

#[cfg(feature = "tds7.4")]
impl<'a> TraceActivityHeaderSpan<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < TraceActivityHeader::LENGTH {
            return Err(DecodeError::InvalidData("".to_string()))
        }
        Ok(Self { bytes })
    }
    #[inline(always)]
    pub fn guid_activity_id(&self) -> &'a [u8; 16] {
        self.bytes[..self.ib_activity_sequence()].try_into().unwrap()
    }
    #[inline(always)]
    const fn ib_activity_sequence(&self) -> usize {
        16
    }
    #[inline(always)]
    pub fn activity_sequence(&self) -> u32 {
        r_u32_le(self.bytes, self.ib_activity_sequence())
    }
}
