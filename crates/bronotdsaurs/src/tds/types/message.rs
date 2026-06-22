use crate::tds::prelude::*;
use core::sync::atomic::{AtomicU8, Ordering};
use alloc::sync::Arc;

/// All server responses use packet header type 0x04 (Tabular Result).
/// See: [MS-TDS] 2.2.3.1.1
pub const SERVER_PACKET_TYPE: u8 = 0x04;

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum ClientMessageType {
    SQLBatch = 0x01,              // 2.2.1.4 SQL Batch
    PreTDS7Login = 0x02,          // 2.2.1.1 Pre-Login
    RemoteProcedureCall = 0x03,   // 2.2.1.6 Remote Procedure Call
    Attention = 0x06,             // 2.2.1.7 Attention
    BulkLoad = 0x07,              // 2.2.1.5 Bulk Load
    FederatedAuthenticationToken = 0x08, // 2.2.1.3 Federated Authentication Token
    TransactionManagerRequest = 0x0e,    // 2.2.1.8 Transaction Manager Request
    TDS7Login = 0x10,             // 2.2.1.2 Login
    SSPI = 0x11,
    PreLogin = 0x12,
}

tds_packet_header!(AttentionHeader, ClientMessageType::Attention);
#[derive(Debug, Clone, Default)]    
pub struct Attention(Arc<AtomicU8>);

impl Attention {
    const IDLE: u8  = 0;
    const REQUESTED: u8 = 1;
    const SENT: u8 = 2;
    pub fn new() -> Self { Self::default() }
    pub fn signal(&self) {
        // Best-effort IDLE -> REQUESTED; a failed CAS just means attention was
        // already requested or sent, which is fine to ignore.
        let _prev = self.0.compare_exchange(Attention::IDLE, Attention::REQUESTED, Ordering::AcqRel, Ordering::Acquire);
    }
    pub(crate) fn take(&self) -> bool {
        self.0.compare_exchange(Attention::REQUESTED,Attention::SENT, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }
    pub(crate) fn sent(&self) -> bool {
        self.0.load(Ordering::Acquire) == Attention::SENT
    }
}