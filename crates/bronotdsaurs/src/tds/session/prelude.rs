// Public transport API
pub use crate::tds::session::error::*;
pub use crate::tds::session::login::LoginReadyStateTransition;
pub use crate::tds::session::observer::Event;
pub use crate::tds::session::prelogin::InitialStateTransition;
pub use crate::tds::session::{
    Session,
    SessionBuffer,
};
pub use crate::tds::session::decode::{
    QueryResult,
    QueryResults,
};
pub use crate::tds::session::request::ClientRequestExecutionTransition;
pub use crate::tds::session::attention::SentAttentionTransition;
#[cfg(feature = "sspi")]
pub use crate::tds::session::sspi::SspiProvider;
#[cfg(feature = "fedauth")]
pub use crate::tds::session::sspi::FedAuthProvider;
pub use crate::tds::session::state::*;
pub use crate::tds::session::routing::Routing;

// Internal transport
pub(in crate::tds::session) use crate::tds::session::timer::Timers;
#[cfg(all(not(feature = "tds8.0"), feature = "tls"))]
pub use crate::tds::session::adaptor::{
    TransportAdaptor,
    TransportAdaptorBuffer,
};
pub use crate::tds::session::traits::{
    Observer,
    Streamer,
};
pub use transport::{
    AsyncReceiver,
    AsyncTransport,
    AsyncSender,
};
#[cfg(feature = "tls")]
pub use transport::tls::TlsHandshaker;

pub const MAX_TDS_PACKET_BYTES: usize = 32767;
/// 1.9 Standards Assignments
pub const DEFAULT_TCP_PORT: u16 = 1433;
#[cfg(feature = "tds8.0")]
pub const TDS_80_ALPN: &[u8] = b"tds/8.0";

pub const DEFAULT_TOKEN_SIZE: u16 = 4096;
