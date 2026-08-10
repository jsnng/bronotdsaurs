use crate::tds::prelude::*;

impl Encoder for QueryNotificationHeader {
    fn encode(&self, buf: &mut [u8]) -> usize {
        let mut cursor: usize = 0;
        let offset = cursor;
        cursor += size_of::<u16>();
        let mut chars = 0u16;
        for char in self.notify_id.encode_utf16() {
            buf[cursor..][..2].copy_from_slice(&char.to_le_bytes());
            cursor += 2;
            chars += 1;
        }
        buf[offset..offset + 2].copy_from_slice(&chars.to_le_bytes());

        let offset = cursor;
        cursor += size_of::<u16>();
        let mut chars = 0u16;
        for char in self.ssb_deployment.encode_utf16() {
            buf[cursor..][..2].copy_from_slice(&char.to_le_bytes());
            cursor += 2;
            chars += 1;
        }
        buf[offset..offset + 2].copy_from_slice(&chars.to_le_bytes());
        wvec!(buf, cursor, self.notify_timeout.to_le_bytes());
        cursor
    }
}

impl Encoder for TransactionDescriptorHeader {
    #[inline]
    fn encode(&self, buf: &mut [u8]) -> usize {
        let mut cursor = 0;
        wint!(buf, cursor, u64, self.transaction_descriptor);
        wint!(buf, cursor, u32, self.outstanding_request_count);
        cursor
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for TraceActivityHeader {
    #[inline]
    fn encode(&self, buf: &mut [u8]) -> usize {
        let mut cursor = 16;
        buf[..cursor].copy_from_slice(&self.guid_activity_id);
        wint!(buf, cursor, u32, self.activity_sequence);
        cursor
    }
}

impl Encoder for AllHeaders {
    fn encode(&self, buf: &mut [u8]) -> usize {
        let mut cursor = 0;
        let ib_all_headers = cursor;
        cursor += size_of::<u32>();
        for header in self.iter() {
            cursor += header.encode(&mut buf[cursor..]);
        }
        let cch_all_headers = (cursor - ib_all_headers) as u32;
        buf[ib_all_headers..ib_all_headers + size_of::<u32>()]
            .copy_from_slice(&cch_all_headers.to_le_bytes());
        cursor
    }
}

impl Encoder for DataStreamHeaderType {
    fn encode(&self, buf: &mut [u8]) -> usize {
        let (ty, size): (u16, usize) = match self {
            DataStreamHeaderType::QueryNotification(x) => (0x0001, x.encode(&mut buf[6..])),
            DataStreamHeaderType::TransactionDescriptor(x) => (0x0002, x.encode(&mut buf[6..])),
            #[cfg(feature = "tds7.4")]
            DataStreamHeaderType::TraceActivity(x) => (0x0003, x.encode(&mut buf[6..])),
        };
        let length = size_of::<u32>() + size_of::<u16>() + size;
        buf[..size_of::<u32>()].copy_from_slice(&(length as u32).to_le_bytes());
        buf[size_of::<u32>()..size_of::<u32>() + size_of::<u16>()].copy_from_slice(&ty.to_le_bytes());
        length
    }
}