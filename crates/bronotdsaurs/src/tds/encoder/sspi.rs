use crate::tds::encoder::traits::MessageEncoder;
use crate::tds::prelude::*;
use crate::tds::session::prelude::*;
use crate::tds::types::prelude::*;
use crate::tds::types::traits::TDSPacketHeader;

#[cfg(feature = "sspi")]
tds_packet_header!(SspiMessageHeader, ClientMessageType::SSPI);
#[cfg(feature = "sspi")]
pub struct SspiMessage {
    pub token: Vec<u8>,
}

#[cfg(feature = "sspi")]
impl MessageEncoder for SspiMessage {
    type Error = EncodeError;
    type Header = SspiMessageHeader;

    fn oneshot(
        &self,
        buf: &mut SessionBuffer,
        header: &mut Self::Header,
    ) -> Result<usize, Self::Error> {
        let mut cursor = SspiMessageHeader::LENGTH;
        buf.writeable()[cursor..cursor + self.token.len()].copy_from_slice(&self.token);
        cursor += self.token.len();
        header.length = cursor as u16;
        buf.writeable()[..SspiMessageHeader::LENGTH].copy_from_slice(&header.as_bytes());
        Ok(cursor)
    }
}

#[cfg(feature = "fedauth")]
tds_packet_header!(FedAuthTokenHeader, ClientMessageType::FederatedAuthenticationToken);
#[cfg(feature = "fedauth")]
pub struct FedAuthToken {
    pub token: Vec<u8>,
}

#[cfg(feature = "fedauth")]
impl MessageEncoder for FedAuthToken {
    type Error = EncodeError;
    type Header = FedAuthTokenHeader;

    fn oneshot(
        &self,
        buf: &mut SessionBuffer,
        header: &mut Self::Header,
    ) -> Result<usize, Self::Error> {
        let mut cursor = FedAuthTokenHeader::LENGTH;
        let data_len = self.token.len() as u32;
        buf.writeable()[cursor..cursor + 4].copy_from_slice(&data_len.to_le_bytes());
        cursor += 4;
        buf.writeable()[cursor..cursor + self.token.len()].copy_from_slice(&self.token);
        cursor += self.token.len();
        header.length = cursor as u16;
        buf.writeable()[..FedAuthTokenHeader::LENGTH].copy_from_slice(&header.as_bytes());
        Ok(cursor)
    }
}
