//! Login State Transitions
use transport::AsyncTransport;
use crate::tds::session::decode::LoginResponse;
use crate::tds::session::prelude::*;
use crate::tds::prelude::*;

pub trait LoginPhase {}
impl LoginPhase for LoginReadyState {}
impl LoginPhase for SpnegoNegotiationState {}

impl<S: LoginPhase, T: AsyncTransport, O: Observer<Event>> Session<S, T, O> {
    pub(in crate::tds::session) async fn receive(&mut self) -> Result<(), SessionError> {
        self.read("LoginAck").await
    }
}

pub enum LoginReadyStateTransition<T, O> {
    LoggedIn {
        session: Session<LoggedInState, T, O>,
    },
    AuthenticationRequired {
        session: Session<LoginReadyState, T, O>,
        errors: Vec<ErrorInfoToken>,
    },
    Routed {
        session: Session<RoutingCompletedState, T, O>,
    },
    SspiContinue {
        session: Session<SpnegoNegotiationState, T, O>,
        challenge: Vec<u8>,
    },
    FedAuthRequired {
        session: Session<FederatedAuthenticationReadyState, T, O>,
        info: Vec<u8>,
    },
}

impl<T: AsyncTransport, O: Observer<Event>> Session<LoginReadyState, T, O> {
    pub async fn transition(
        mut self,
        login7: Login7Packet,
    ) -> Result<LoginReadyStateTransition<T, O>, SessionError> {
        self.send(login7).await?;
        self.notify(Event::Login7Sent);
        self.receive().await?;

        let res = LoginResponse::new(&self.buffer.readable()[Login7Header::LENGTH..])?;

        if let Some((protocol, host, port)) = res.routing {
            self.notify(Event::StateTransition {
                from: "LoginReadyState",
                to: "RoutingCompletedState",
            });
            return Ok(LoginReadyStateTransition::Routed {
                session: self.with_state(RoutingCompletedState { protocol, host, port }),
            });
        }

        if !res.login_ack {
            if let Some(challenge) = res.sspi_challenge {
                self.notify(Event::StateTransition {
                    from: "LoginReadyState",
                    to: "SpnegoNegotiationState",
                });
                return Ok(LoginReadyStateTransition::SspiContinue {
                    session: self.with_state(SpnegoNegotiationState),
                    challenge,
                });
            }

            self.notify(Event::StateTransition {
                from: "LoginReadyState",
                to: "LoginReadyState",
            });
            return Ok(LoginReadyStateTransition::AuthenticationRequired {
                session: self.with_state(LoginReadyState),
                errors: res.errors,
            });
        }

        if let Some(size) = res.packet_size {
            self.buffer.set_buffer_maximum_size(size)?;
        }

        self.notify(Event::StateTransition {
            from: "LoginReadyState",
            to: "LoggedInState",
        });

        Ok(LoginReadyStateTransition::LoggedIn {
            session: self.with_state(
                LoggedInStateBuilder::default()
                    .transaction_descriptor(res.transaction_descriptor)
                    .build()
                    .unwrap(),
            ),
        })
    }
}

#[cfg(feature = "sspi")]
impl<T: AsyncTransport, O: Observer<Event>> Session<SpnegoNegotiationState, T, O> {
    pub async fn transition_spnego<P: crate::tds::session::sspi::SspiProvider>(
        mut self,
        mut provider: P,
        mut challenge: Vec<u8>,
    ) -> Result<LoginReadyStateTransition<T, O>, SessionError> {
        loop {
            let token = provider
                .next(&challenge)
                .map_err(|e| SessionError::MappedError(alloc::format!("sspi provider: {:?}", e)))?;
            let Some(token) = token else {
                return Err(SessionError::Unimplemented);
            };
            self.send(crate::tds::session::sspi::SspiMessage { token }).await?;
            self.receive().await?;

            let res = LoginResponse::new(&self.buffer.readable()[Login7Header::LENGTH..])?;

            if res.login_ack {
                return Ok(LoginReadyStateTransition::LoggedIn {
                    session: self.with_state(
                        LoggedInStateBuilder::default()
                            .transaction_descriptor(res.transaction_descriptor)
                            .build()
                            .unwrap(),
                    ),
                });
            }

            match res.sspi_challenge {
                Some(c) => challenge = c,
                None => {
                    return Ok(LoginReadyStateTransition::AuthenticationRequired {
                        session: self.with_state(LoginReadyState),
                        errors: res.errors,
                    });
                }
            }
        }
    }
}

#[cfg(feature = "fedauth")]
impl<T: AsyncTransport, O: Observer<Event>> Session<FederatedAuthenticationReadyState, T, O> {
    pub async fn transition_fedauth<P: crate::tds::session::sspi::FedAuthProvider>(
        self,
        mut _provider: P,
        _info: Vec<u8>,
    ) -> Result<LoginReadyStateTransition<T, O>, SessionError> {
        Err(SessionError::Unimplemented)
    }
}
