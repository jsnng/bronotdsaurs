use crate::tds::prelude::*;
use crate::tds::session::decode::decode_token_stream;
use crate::tds::session::prelude::*;

pub enum ClientRequestExecutionTransition<T, O> {
    Completed {
        session: Session<LoggedInState, T, O>,
        results: QueryResults,
    },
    Routed {
        session: Session<RoutingCompletedState, T, O>,
    },
    Error {
        session: Session<LoggedInState, T, O>,
        errors: Vec<ErrorInfoToken>,
    },
}

impl<T: AsyncTransport, O: Observer<Event>> Session<LoggedInState, T, O> {
    pub fn transaction_descriptor(&self) -> u64 {
        self.state.transaction_descriptor
    }

    pub async fn execute<Msg>(
        mut self,
        msg: Msg,
    ) -> Result<Session<ClientRequestExecutionState, T, O>, SessionError>
    where
        Msg: MessageEncoder<Error = EncodeError>,
        Msg::Header: Default,
    {
        self.send(msg).await?;
        self.notify(Event::StateTransition {
            from: "LoggedInState",
            to: "ClientRequestExecutionState",
        });
        Ok(Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: ClientRequestExecutionState {
                transaction_descriptor: self.state.transaction_descriptor,
            },
        })
    }
}

impl<T: AsyncTransport, O: Observer<Event>> Session<ClientRequestExecutionState, T, O> {
    pub fn transaction_descriptor(&self) -> u64 {
        self.state.transaction_descriptor
    }

    pub async fn receive<M, F>(
        mut self,
        on_col_metadata: M,
        on_row: F,
    ) -> Result<ClientRequestExecutionTransition<T, O>, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        AsyncTransport::set_read_timeout(&mut self.stream, self.timers.request)
            .map_err(|_| SessionError::transport_read_error())?;

        let output = decode_token_stream(&mut self.stream, on_col_metadata, on_row).await?;

        self.notify(Event::BytesReceived {
            heading: "QueryResponse",
            len: output.results.results.len(),
        });

        let mut transaction_descriptor = self.state.transaction_descriptor;
        if let Some(td) = output.transaction_descriptor {
            transaction_descriptor = td;
        }

        if let Some((protocol, host, port)) = output.routing {
            self.notify(Event::StateTransition {
                from: "ClientRequestExecutionState",
                to: "RoutingCompletedState",
            });
            return Ok(ClientRequestExecutionTransition::Routed {
                session: Session {
                    stream: self.stream,
                    observer: self.observer,
                    timers: self.timers,
                    buffer: self.buffer,
                    state: RoutingCompletedState {
                        protocol,
                        host,
                        port,
                    },
                },
            });
        }

        let errors = output.results.errors();
        self.notify(Event::StateTransition {
            from: "ClientRequestExecutionState",
            to: "LoggedInState",
        });
        let session = Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: LoggedInStateBuilder::default()
                .transaction_descriptor(transaction_descriptor)
                .build()
                .unwrap(),
        };
        if !errors.is_empty() {
            Ok(ClientRequestExecutionTransition::Error { session, errors })
        } else {
            Ok(ClientRequestExecutionTransition::Completed {
                session,
                results: output.results,
            })
        }
    }
}
