//! Login State Transitions
use transport::AsyncTransport;
use crate::tds::decoder::stream::{NoContextStep, TokenDecoder};
use crate::tds::session::prelude::*;
use crate::tds::prelude::*;

impl<T: AsyncTransport, O: Observer<Event>> AsyncReceiver<T> for Session<LoginReadyState, T, O> {
    type Error = SessionError;
    type Output<'a>
        =  &'a [u8] where Self: 'a;

    async fn receive(&mut self) -> Result<(), Self::Error> {
        self.buffer.reset();
        self.notify(Event::Log(alloc::format!(
            "[login::receive] reading header ({} bytes)",
            Login7Header::LENGTH
        )));

        let mut head = 0;
        while head < Login7Header::LENGTH {
            let n = self
                .stream
                .read(&mut self.buffer.writeable()[head..Login7Header::LENGTH])
                .await
                .map_err(|_| SessionError::transport_read_error())?;
            if n == 0 {
                return Err(SessionError::ServerClosedTransportConnection);
            }
            head += n;
        }
        self.buffer.tail(Login7Header::LENGTH)?;

        let buf = self.buffer.readable();
        let length = r_u16_be(buf, 2) as usize;
        if length < Login7Header::LENGTH {
            return Err(SessionError::InvalidPacketLength { got: length, expected: Login7Header::LENGTH });
        }
        let payload_length = length - Login7Header::LENGTH;

        self.notify(Event::Log(alloc::format!("[login::receive] header ok — packet length={length}, reading payload ({payload_length} bytes)")));
        let mut reading = 0;
        while reading < payload_length {
            let n = self
                .stream
                .read(&mut self.buffer.writeable()[reading..payload_length])
                .await
                .map_err(|_| SessionError::transport_read_error())?;
            self.notify(Event::Log(alloc::format!(
                "[login::receive] payload read: n={n} (total={}/{payload_length})",
                reading + n
            )));
            if n == 0 {
                return Err(SessionError::ServerClosedTransportConnection);
            }
            reading += n;
        }
        self.buffer.tail(payload_length)?;

        self.notify(Event::BytesReceived {
            heading: "LoginAck",
            len: self.buffer.readable().len(),
        });

        Ok(())
    }

    fn output(&self) -> Result<Self::Output<'_>, Self::Error> {
        Ok(&self.buffer.readable()[Login7Header::LENGTH..])
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
}

impl<T: AsyncTransport, O: Observer<Event>> Session<LoginReadyState, T, O> {
    pub async fn transition(
        mut self,
        login7: Login7Packet,
    ) -> Result<LoginReadyStateTransition<T, O>, SessionError> {
        self.send(login7).await?;
        self.notify(Event::Login7Sent);
        self.receive().await?;

        let readable = &self.buffer.readable()[Login7Header::LENGTH..];
        let mut errors: Vec<ErrorInfoToken> = Vec::with_capacity(4);
        let mut infos: Vec<ErrorInfoToken> = Vec::with_capacity(4);

        let mut has_change_maximum_size = false;
        let mut default_maximum_size: usize = 4096;

        let mut has_login_ack_span = false;

        let mut decoder = TokenDecoder::new(readable);
        loop {
            match decoder.advance() {
                Some(NoContextStep::EnvChange(x, next)) => {
                    #[allow(clippy::single_match)]
                    match x.ty() {
                        Some(EnvChangeType::PacketSize) => {
                            has_change_maximum_size = true;
                            let data = x.env_value_data();
                            let chars = data.first().copied().unwrap_or(0) as usize;
                            let needed = 1 + chars.saturating_mul(2);
                            default_maximum_size = data.get(1..needed)
                                .and_then(|s| {
                                    s.chunks_exact(2)
                                        .map(|c| c[0])
                                        .try_fold(0usize, |acc, b| {
                                            if b.is_ascii_digit() {
                                                acc.checked_mul(10)
                                                    .and_then(|v| v.checked_add((b - b'0') as usize))
                                            } else {
                                                None
                                            }
                                        })
                                })
                                .filter(|n| (512..=32_768).contains(n))
                                .unwrap_or(4096);
                        }
                        _ => {}
                    }
                    decoder = next;
                }
                Some(NoContextStep::ServerError(x, next)) => {
                    errors.push(x.own());
                    decoder = next;
                }
                Some(NoContextStep::Info(x, next)) => {
                    infos.push(x.own());
                    decoder = next;
                }
                Some(NoContextStep::LoginAck(_, next)) => {
                    has_login_ack_span = true;
                    decoder = next;
                }
                #[cfg(feature = "tds7.4")]
                Some(NoContextStep::FeatureExtAck(_, next)) => decoder = next,
                #[cfg(feature = "tds7.4")]
                Some(NoContextStep::SessionState(_, next)) => decoder = next,
                Some(NoContextStep::Sspi(_, next)) => decoder = next,
                Some(NoContextStep::Done(_, _)) | Some(NoContextStep::Error(_)) | None => break,
                _ => unreachable!()
            }
        }

        if !has_login_ack_span {
            self.notify(Event::StateTransition {
                from: "LoginReadyState",
                to: "LoginReadyState",
            });
            return Ok(LoginReadyStateTransition::AuthenticationRequired {
                session: Self {
                    stream: self.stream,
                    observer: self.observer,
                    timers: self.timers,
                    buffer: self.buffer,
                    state: self.state,
                },
                errors,
            });
        }

        if has_change_maximum_size {
            self.buffer.set_buffer_maximum_size(default_maximum_size)?;
        }

        self.notify(Event::StateTransition {
            from: "LoginReadyState",
            to: "LoggedInState",
        });

        Ok(LoginReadyStateTransition::LoggedIn {
            session: Session {
                stream: self.stream,
                observer: self.observer,
                buffer: self.buffer,
                timers: self.timers,
                state: LoggedInStateBuilder::default()
                    .transaction_descriptor(0)
                    .build()
                    .unwrap(),
            },
        })
    }
}
