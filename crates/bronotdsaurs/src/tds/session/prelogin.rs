//! InitialState/PreLoginState State Transitions
use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[cfg(feature = "std")]
use tracing::debug;

/// Marker trait [`PreLoginPhase`] due to different TLS negotiation requirements.
/// TDS 7.x: PreLogin -> TLS negotiation -> Login
/// TDS 8.0: TLS negotiation -> PreLogin -> Login
///
/// Implemented by [`InitialState`] for 7.x and [`PreLoginReadyState`] for 8.0
pub trait PreLoginPhase {}
#[cfg(not(feature = "tds8.0"))]
impl PreLoginPhase for InitialState {}
#[cfg(feature = "tds8.0")]
impl PreLoginPhase for PreLoginReadyState {}

impl<S: PreLoginPhase, T: AsyncTransport, O: Observer<Event>> AsyncReceiver<T> for Session<S, T, O> {
    type Error = SessionError;
    type Output<'a>
        =  PreLoginSpan<'a> where Self: 'a;

    async fn receive(&mut self) -> Result<(), Self::Error> {
        self.read("PreLogin").await
    }
    fn output(&self) -> Result<Self::Output<'_>, Self::Error> {
        PreLoginSpan::new(self.buffer.readable()).map_err(SessionError::from)
    }
}

#[derive(Debug)]
pub enum InitialStateTransition<T, O> {
    #[cfg(all(not(feature = "tds8.0"), feature = "tls"))]
    TlsSslNegotiation(Session<TlsSslNegotiationState, T, O>),
    LoginReady(Session<LoginReadyState, T, O>),
    #[cfg(feature = "tds8.0")]
    TlsNegotiation(Session<TlsNegotiationState, T, O>),
}

#[cfg(not(feature = "tds8.0"))]
impl<T: AsyncTransport, O: Observer<Event>> Session<InitialState, T, O> {
    pub async fn transition(
        mut self,
        prelogin: PreLoginPacket,
    ) -> Result<InitialStateTransition<T, O>, SessionError> {
        AsyncTransport::set_read_timeout(&mut self.stream, self.timers.connection)
            .map_err(|_| SessionError::transport_read_error())?;
        AsyncTransport::set_write_timeout(&mut self.stream, self.timers.connection)
            .map_err(|_| SessionError::transport_write_error())?;
        self.send(prelogin).await?;
        self.notify(Event::PreLoginSent);

        self.receive().await?;

        let bytes = PreLoginSpan::populate(self.buffer.readable())?
            .encryption()
            .unwrap_or(PreLoginEncryptionOptions::NotSupported as u8);
        self.notify(Event::PreLoginReceived);

        #[cfg(feature = "std")]
        debug!("Server encryption byte = 0x{:02x}", bytes);

        let encryption: PreLoginEncryptionOptions = bytes
            .try_into()?;

        #[cfg(feature = "std")]
        debug!("Parsed as {:?}", encryption);

        match encryption {
            PreLoginEncryptionOptions::Off | PreLoginEncryptionOptions::NotSupported => {
                self.notify(Event::StateTransition {
                    from: "Initial",
                    to: "LoginReady",
                });
                Ok(InitialStateTransition::LoginReady(Session {
                    stream: self.stream,
                    observer: self.observer,
                    buffer: self.buffer,
                    timers: self.timers,
                    state: LoginReadyState,
                }))
            }
            PreLoginEncryptionOptions::On | PreLoginEncryptionOptions::Required => {
                #[cfg(feature = "tls")]
                {
                    self.notify(Event::StateTransition {
                        from: "Initial",
                        to: "TlsSslNegotiation",
                    });
                    Ok(InitialStateTransition::TlsSslNegotiation(Session {
                        stream: self.stream,
                        observer: self.observer,
                        buffer: self.buffer,
                        timers: self.timers,
                        state: TlsSslNegotiationState,
                    }))
                }
                #[cfg(not(feature = "tls"))]
                Err(SessionError::Unimplemented)
            }
            _ => Err(DecodeError::InvalidField(format!("PreLogin response: unsupported encryption option: {:?}", encryption)).into()),
        }
    }
}

#[cfg(feature = "tds8.0")]
impl<T: AsyncTransport, O: Observer<Event>> Session<InitialState, T, O> {
    pub fn transition(mut self) -> Result<InitialStateTransition<T, O>, SessionError> {
        AsyncTransport::set_read_timeout(&mut self.stream, self.timers.connection)
            .map_err(|_| SessionError::transport_read_error())?;
        AsyncTransport::set_write_timeout(&mut self.stream, self.timers.connection)
            .map_err(|_| SessionError::transport_write_error())?;

        Ok(InitialStateTransition::TlsNegotiation(Session {
                        stream: self.stream,
                        observer: self.observer,
                        buffer: self.buffer,
                        timers: self.timers,
                        state: TlsNegotiationState,
        }))
    }
}

#[cfg(all(not(feature = "tds8.0"), feature = "tls"))]
impl<T: AsyncTransport, O: Observer<Event>> Session<TlsSslNegotiationState, T, O> {
    pub async fn transition<P, H, F>(
        self,
        server_name: &str,
        handshaker: H,
        factory: F,
    ) -> Result<Session<LoginReadyState, P, O>, SessionError>
    where
        P: AsyncTransport,
        H: TlsHandshaker,
        H::HandshakeError: core::fmt::Debug,
        F: FnOnce(T, H::Connection) -> P,
    {
        let Session {
            mut stream,
            mut observer,
            timers,
            buffer,
            ..
        } = self;

        let connection = {
            let mut adaptor = TransportAdaptor {
                transport: &mut stream,
                reader: TransportAdaptorBuffer::default(),
            };
            handshaker
                .handshake(server_name, &mut adaptor)
                .await
                .map_err(|e| {
                    SessionError::MappedError(alloc::format!("TLS handshake failed {:?}", e))
                })?
        };

        observer.on(&Event::StateTransition {
            from: "TlsSslNegotiation",
            to: "LoginReady",
        });
        Ok(Session {
            stream: factory(stream, connection),
            observer,
            timers,
            buffer,
            state: LoginReadyState,
        })
    }
}

#[cfg(feature = "tds8.0")]
impl<T: AsyncTransport, O: Observer<Event>> Session<TlsNegotiationState, T, O> {
    pub async fn transition<P, H, F>(
        self,
        server_name: &str,
        handshaker: H,
        factory: F,
    ) -> Result<Session<PreLoginReadyState, P, O>, SessionError>
    where
        P: AsyncTransport,
        H: TlsHandshaker,
        H::HandshakeError: core::fmt::Debug,
        F: FnOnce(T, H::Connection) -> P,
    {
        let Session {
            mut stream,
            mut observer,
            timers,
            buffer,
            ..
        } = self;

        let connection =
            handshaker
                .handshake(server_name, &mut stream)
                .await
                .map_err(|e| {
                    SessionError::MappedError(alloc::format!("TLS handshake failed {:?}", e))
                })?;

        observer.on(&Event::StateTransition {
            from: "TlsNegotiation",
            to: "PreLoginReady",
        });
        Ok(Session {
            stream: factory(stream, connection),
            observer,
            timers,
            buffer,
            state: PreLoginReadyState,
        })
    }
}


#[cfg(feature = "tds8.0")]
impl<T: AsyncTransport, O: Observer<Event>> Session<PreLoginReadyState, T, O> {
    pub async fn transition(
        mut self,
        prelogin: PreLoginPacket,
    ) -> Result<Session<LoginReadyState, T, O>, SessionError> {
        self.send(prelogin).await?;
        self.notify(Event::PreLoginSent);

        self.receive().await?;
        self.notify(Event::PreLoginReceived);

        self.notify(Event::StateTransition {
            from: "PreLoginReady",
            to: "LoginReady",
        });
        Ok(Session {
            stream: self.stream,
            observer: self.observer,
            buffer: self.buffer,
            timers: self.timers,
            state: LoginReadyState,
        })
    }
}