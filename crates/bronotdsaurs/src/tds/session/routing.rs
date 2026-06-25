use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Routing {
    pub protocol: u8,
    pub host: String,
    pub port: u16,
}

pub fn parse_routing(span: &EnvChangeSpan<'_>) -> Result<Routing, SessionError> {
    let bytes = span.bytes;
    if bytes.len() < 11 {
        return Err(SessionError::DecodeError(DecodeError::InvalidLength(
            format!("routing env change too short: {} bytes", bytes.len()),
        )));
    }
    let protocol = bytes[6];
    if protocol != 0 {
        return Err(SessionError::Unimplemented);
    }
    let port = u16::from_le_bytes([bytes[7], bytes[8]]);
    let chars = u16::from_le_bytes([bytes[9], bytes[10]]) as usize;
    let start = 11;
    let end = start + chars * 2;
    if end > bytes.len() {
        return Err(SessionError::DecodeError(DecodeError::InvalidLength(
            format!("routing alternate server overruns token: end={end} len={}", bytes.len()),
        )));
    }
    let host: String = core::char::decode_utf16(
        bytes[start..end]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]])),
    )
    .map(|r| r.unwrap_or(core::char::REPLACEMENT_CHARACTER))
    .collect();
    Ok(Routing { protocol, host, port })
}

impl<T: AsyncTransport, O: Observer<Event>> Session<RoutingCompletedState, T, O> {
    pub fn host(&self) -> &str {
        &self.state.route.host
    }

    pub fn port(&self) -> u16 {
        self.state.route.port
    }

    pub fn protocol(&self) -> u8 {
        self.state.route.protocol
    }

    pub fn into_route(self) -> (String, u16) {
        (self.state.route.host, self.state.route.port)
    }

    pub fn disconnect(self) -> Session<FinalState, T, O> {
        Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: FinalState,
        }
    }
}

impl<T: AsyncTransport, O: Observer<Event>> Session<FinalState, T, O> {
    pub fn into_transport(self) -> T {
        self.stream
    }
}