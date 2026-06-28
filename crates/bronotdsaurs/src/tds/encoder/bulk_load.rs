use crate::tds::prelude::*;
use transport::AsyncTransport;

pub struct BulkLoadWriter {
    packet_size: usize,
    packet_id: u8,
}

impl BulkLoadWriter {
    pub fn new(packet_size: usize) -> Self {
        Self { packet_size, packet_id: 0 }
    }

    pub async fn write<T: AsyncTransport>(
        &mut self,
        transport: &mut T,
        payload: &[u8],
        eom: bool,
    ) -> Result<(), T::Error> {
        let max_payload_size = self.packet_size.saturating_sub(8).max(1);
        let mut chunks = payload.chunks(max_payload_size).peekable();
        while let Some(chunk) = chunks.next() {
            let is_last = chunks.peek().is_none();
            let status = if eom && is_last {
                MessageStateStatus::EndOfMessage as u8
            } else {
                MessageStateStatus::Normal as u8
            };
            let length = (8 + chunk.len()) as u16;
            let header = [
                ClientMessageType::BulkLoad as u8,
                status,
                (length >> 8) as u8,
                length as u8,
                0,
                0,
                self.packet_id,
                0,
            ];
            self.packet_id = self.packet_id.wrapping_add(1);
            Self::drain(transport, &header).await?;
            Self::drain(transport, chunk).await?;
        }
        Ok(())
    }

    async fn drain<T: AsyncTransport>(transport: &mut T, mut buf: &[u8]) -> Result<(), T::Error> {
        while !buf.is_empty() {
            let n = transport.write(buf).await?;
            if n == 0 {
                break;
            }
            buf = &buf[n..];
        }
        Ok(())
    }
}

pub struct BulkLoad {
    col_metadata: ColMetaDataToken,
    writer: BulkLoadWriter,
    buf: Vec<u8>,
}

impl BulkLoad {
    pub async fn new<T: AsyncTransport>(
        transport: &mut T,
        col_metadata: ColMetaDataToken,
        packet_size: usize,
    ) -> Result<Self, T::Error> {
        let mut writer = BulkLoadWriter::new(packet_size);
        writer.write(transport, &col_metadata.as_bytes(), false).await?;
        Ok(Self { col_metadata, writer, buf: Vec::new() })
    }

    pub fn push_row(&mut self, columns: &[Vec<u8>]) {
        self.buf.clear();
        self.buf.push(DataTokenType::Row as u8);
        for (bytes, item) in columns.iter().zip(self.col_metadata.column_data.iter()) {
            match item.type_info.dtype_max_len {
                Some(TypeInfoVarLen::Byte(_)) => self.buf.push(bytes.len() as u8),
                Some(TypeInfoVarLen::Ushort(_)) => self.buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes()),
                Some(TypeInfoVarLen::Long(_)) => self.buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes()),
                None => {}
            }
            self.buf.extend_from_slice(bytes);
        }
    }

    #[cfg(feature = "tds7.3b")]
    pub fn push_nbc_row(&mut self, columns: &[Option<Vec<u8>>]) {
        self.buf.clear();
        self.buf.push(DataTokenType::NbcRow as u8);
        let bitmap = self.col_metadata.column_data.len().div_ceil(8);
        let cursor = self.buf.len();
        self.buf.resize(cursor + bitmap, 0);
        let mut idx = cursor;
        let mut mask = 1u8;
        for (col, item) in columns.iter().zip(self.col_metadata.column_data.iter()) {
            match col {
                None => self.buf[idx] |= 1 << mask,
                Some(bytes) => {
                    match item.type_info.dtype_max_len {
                        Some(TypeInfoVarLen::Byte(_)) => self.buf.push(bytes.len() as u8),
                        Some(TypeInfoVarLen::Ushort(_)) => self.buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes()),
                        Some(TypeInfoVarLen::Long(_)) => self.buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes()),
                        None => {}
                    }
                    self.buf.extend_from_slice(bytes);
                }
            }
            mask = mask.rotate_left(1);
            idx += (mask == 1) as usize;
        }
    }

    pub async fn flush<T: AsyncTransport>(&mut self, transport: &mut T) -> Result<(), T::Error> {
        self.writer.write(transport, &self.buf, false).await
    }

    pub async fn done<T: AsyncTransport>(
        mut self,
        transport: &mut T,
        done_token: DoneToken,
    ) -> Result<(), T::Error> {
        self.writer.write(transport, &done_token.as_bytes(), true).await
    }
}
