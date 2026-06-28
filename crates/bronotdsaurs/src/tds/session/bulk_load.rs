use crate::tds::encoder::bulk_load::BulkLoad;
use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

impl<T: AsyncTransport, O: Observer<Event>> Session<LoggedInState, T, O> {
    pub async fn bulk_load(
        mut self,
        col_metadata: ColMetaDataToken,
    ) -> Result<Session<BulkLoadState, T, O>, SessionError> {
        let packet_size = self.buffer.buffer_size();
        let bulk = BulkLoad::new(&mut self.stream, col_metadata, packet_size)
            .await
            .map_err(|_| SessionError::transport_write_error())?;
        self.notify(Event::StateTransition {
            from: "LoggedInState",
            to: "BulkLoadState",
        });
        Ok(Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: BulkLoadState {
                transaction_descriptor: self.state.transaction_descriptor,
                bulk,
            },
        })
    }
}

impl<T: AsyncTransport, O: Observer<Event>> Session<BulkLoadState, T, O> {
    pub fn transaction_descriptor(&self) -> u64 {
        self.state.transaction_descriptor
    }

    pub async fn push(&mut self, columns: &[Vec<u8>]) -> Result<(), SessionError> {
        self.state.bulk.push_row(columns);
        self.state
            .bulk
            .flush(&mut self.stream)
            .await
            .map_err(|_| SessionError::transport_write_error())
    }

    pub async fn done(
        mut self,
        done_token: DoneToken,
    ) -> Result<Session<ClientRequestExecutionState, T, O>, SessionError> {
        let transaction_descriptor = self.state.transaction_descriptor;
        self.notify(Event::StateTransition {
            from: "BulkLoadState",
            to: "ClientRequestExecutionState",
        });
        self.state
            .bulk
            .done(&mut self.stream, done_token)
            .await
            .map_err(|_| SessionError::transport_write_error())?;
        Ok(Session {
            stream: self.stream,
            observer: self.observer,
            timers: self.timers,
            buffer: self.buffer,
            state: ClientRequestExecutionState {
                transaction_descriptor,
            },
        })
    }
}
