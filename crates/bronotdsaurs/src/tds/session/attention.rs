use crate::tds::prelude::*;
use crate::tds::session::decode::decode_token_stream;
use crate::tds::session::prelude::*;

pub enum SentAttentionTransition<T, O> {
    Acknowledged {
        session: Session<LoggedInState, T, O>,
        results: QueryResults,
    },
    Error {
        session: Session<LoggedInState, T, O>,
        errors: Vec<ErrorInfoToken>,
    },
}

impl<T: AsyncTransport, O: Observer<Event>> Session<ClientRequestExecutionState, T, O> {
    pub async fn attention(
        mut self,
    ) -> Result<Session<SentAttentionState, T, O>, SessionError> {
        self.send(Attention::new()).await?;
        self.notify(Event::StateTransition {
            from: "ClientRequestExecutionState",
            to: "SentAttentionState",
        });
        Ok(Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: SentAttentionState {
                transaction_descriptor: self.state.transaction_descriptor,
            },
        })
    }
}

impl<T: AsyncTransport, O: Observer<Event>> Session<SentAttentionState, T, O> {
    pub async fn receive<M, F>(
        mut self,
        on_col_metadata: M,
        on_row: F,
    ) -> Result<SentAttentionTransition<T, O>, SessionError>
    where
        M: FnMut(&ColMetaDataOwned),
        F: for<'r> FnMut(&ColMetaDataOwned, &'r [u8]),
    {
        AsyncTransport::set_read_timeout(&mut self.stream, self.timers.cancel)
            .map_err(|_| SessionError::transport_read_error())?;

        let output = decode_token_stream(&mut self.stream, on_col_metadata, on_row).await?;

        self.notify(Event::StateTransition {
            from: "SentAttentionState",
            to: "LoggedInState",
        });

        let errors = output.results.errors();
        let session = Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: LoggedInStateBuilder::default()
                .transaction_descriptor(self.state.transaction_descriptor)
                .build()
                .unwrap(),
        };
        if !errors.is_empty() {
            Ok(SentAttentionTransition::Error { session, errors })
        } else {
            Ok(SentAttentionTransition::Acknowledged {
                session,
                results: output.results,
            })
        }
    }
}
