use derive_builder::Builder;

#[derive(Debug, Default)]
pub struct InitialState;
#[cfg(all(not(feature = "tds8.0"), feature = "tls"))]
#[derive(Debug, Default)]
pub struct TlsSslNegotiationState;
#[cfg(feature = "tds8.0")]
#[derive(Debug, Default)]
pub struct TlsNegotiationState;
#[cfg(feature = "tds8.0")]
#[derive(Debug, Default)]
pub struct PreLoginReadyState;
#[derive(Debug, Default)]
pub struct MARS;
#[derive(Debug, Default)]
pub struct LoginReadyState;
#[derive(Debug, Default)]
pub struct SpnegoNegotiationState;
#[derive(Debug, Default)]
pub struct FederatedAuthenticationReadyState;
#[derive(Debug, Clone, Copy, Default, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct LoggedInState {
    pub transaction_descriptor: u64,
}
#[derive(Debug)]
pub struct ClientRequestExecutionState {
    pub transaction_descriptor: u64,
}
#[derive(Debug, Default)]
pub struct SentAttentionState {
    pub transaction_descriptor: u64,
}
#[derive(Debug, Default)]
pub struct RoutingCompletedState {
    pub protocol: u8,
    pub host: alloc::string::String,
    pub port: u16,
}
#[derive(Debug, Default)]
pub struct FinalState;
