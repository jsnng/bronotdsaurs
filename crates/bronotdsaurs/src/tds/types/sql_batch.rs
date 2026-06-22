use crate::tds::prelude::*;

tds_packet_header!(SQLBatchHeader, ClientMessageType::SQLBatch);

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct SQLBatch {
    pub(crate) all_headers: AllHeaders,
    #[cfg(feature = "tds8.0")]
    pub(crate) enclave_package: u8,
    pub(crate) sql_text: String,
}

#[cfg(feature = "tds8.0")]
pub struct EnclavePackage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tds::session::prelude::SessionBuffer;

    #[test]
    fn test_sql_batch_encode() {
        let bytes = [
            0x01, 0x01, 0x00, 0x5C, 0x00, 0x00, 0x01, 0x00, 0x16, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00,
            0x73, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x65, 0x00, 0x63, 0x00, 0x74, 0x00, 0x20, 0x00, 0x27, 0x00,
            0x66, 0x00, 0x6F, 0x00, 0x6F, 0x00, 0x27, 0x00, 0x20, 0x00, 0x61, 0x00, 0x73, 0x00, 0x20, 0x00,
            0x27, 0x00, 0x62, 0x00, 0x61, 0x00, 0x72, 0x00, 0x27, 0x00, 0x0A, 0x00, 0x20, 0x00, 0x20, 0x00,
            0x20, 0x00, 0x20, 0x00, 0x20, 0x00, 0x20, 0x00, 0x20, 0x00, 0x20, 0x00,
        ];

        let all_headers = AllHeaders::new(alloc::vec![
            DataStreamHeaderType::TransactionDescriptor(TransactionDescriptorHeader {
                transaction_descriptor: 0x0100_0000_0000_0000,
                outstanding_request_count: 0,
            }),
        ]);

        let sql_batch = SQLBatchBuilder::default()
            .all_headers(all_headers)
            .sql_text(String::from("\nselect 'foo' as 'bar'\n        "))
            .build()
            .unwrap();

        let mut header = SQLBatchHeader {
            packet_id: 0x01,
            ..Default::default()
        };

        let mut buffer = SessionBuffer::default();
        let n = sql_batch
            .oneshot(&mut buffer, &mut header)
            .expect("");
        let _n = buffer.tail(n);

        assert_eq!(&buffer.readable()[..n], &bytes[..]);
    }
}