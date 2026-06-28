//! 2.2.6.5 PreLogin Types
use crate::tds::prelude::*;

#[cfg(kani)]
extern crate kani;

tds_packet_header!(PreLoginHeader, ClientMessageType::PreLogin);
crate::span!(PreLoginOptionSpan,);

#[derive(Debug, Clone, Copy, Default)]
pub struct PreLoginSpan<'a> {
    bytes: &'a [u8],
    pub(crate) version: Option<&'a [u8]>,
    pub(crate) encryption: Option<&'a [u8]>,
    pub(crate) inst_opt: Option<&'a [u8]>,
    pub(crate) thread_id: Option<&'a [u8]>,
    pub(crate) mars: Option<&'a [u8]>,
    payload: Option<&'a [u8]>,
}

impl<'a> PreLoginSpan<'a> {
    pub fn version(&self) -> [u8; 6] {
        self.version
            .and_then(|x| x.try_into().ok())
            .unwrap_or([0u8; 6])
    }
    pub fn encryption(&self) -> Option<u8> {
        self.encryption.map(|x| x[0])
    }
    pub fn inst_opt(&self) -> Option<&[u8]> {
        self.inst_opt
    }
    pub fn thread_id(&self) -> Option<u32> {
        self.thread_id
            .and_then(|x| x.try_into().ok())
            .map(u32::from_be_bytes)
    }
    pub fn mars(&self) -> Option<u8> {
        self.mars.map(|x| x[0])
    }

    // #[cfg(not(feature = "unsafe"))]
    // Pre: `PLOptionType::Terminator` exists in the arary.
    #[cfg_attr(kani, kani::requires(self.bytes.len() >= PreLoginHeader::LENGTH &&
        self.bytes[0..].iter().any(|&x| x == PLOptionType::Terminator)
    ))]
    // Post: If `PLOptionType::Terminator` exists at some position i,
    // then the length of the u8 array == self.bytes.len() - (i + 1)
    #[cfg_attr(kani, kani::ensures(|x: &&[u8]| {
        if let Some(i) = self.bytes[PreLoginHeader::LENGTH..]
            .iter()
            .position(|&b| b == (PLOptionType::Terminator))
        {
            x.len() == self.bytes.len() - (PreLoginHeader::LENGTH + i + PreLoginPacket::TERMINATOR_OPT_LENGTH)
        } else {
            x.is_empty()
        }
    }
    ))]
    // Performs a byte slice from the terminator option token.
    // An empty [u8] is returned if the terminator token is at the end of the array.
    pub fn payload(&self) -> &'a [u8] {
        self.payload.unwrap_or(&[])
    }
}

impl PreLoginPacket {
    pub fn version(&self) -> [u8; 6] {
        self.version
    }
    pub fn encryption(&self) -> Option<u8> {
        self.encryption
    }
    pub fn inst_opt(&self) -> Option<&[u8]> {
        self.inst_opt.as_deref()
    }
    pub fn thread_id(&self) -> Option<u32> {
        self.thread_id
    }
    pub fn mars(&self) -> Option<u8> {
        self.mars
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(no_std, setter(strip_option), default)]
pub struct PreLoginPacket {
    pub(crate) version: [u8; 6],
    pub(crate) encryption: Option<u8>,
    pub(crate) inst_opt: Option<Vec<u8>>,
    pub(crate) thread_id: Option<u32>,
    pub(crate) mars: Option<u8>,
    payload: Vec<u8>,
}

impl PreLoginPacket {
    pub const OPT_LENGTH: usize = 5;
    pub const TERMINATOR_OPT_LENGTH: usize = 1;
}

#[rustfmt::skip]
impl<'a> PreLoginSpan<'a> {
    #[inline(always)]
    // Post: returns `Some` iff there are at least PreLoginHeader::LENGTH bytes and
    // type = ClientMessageType::PreLogin (0x12) or SERVER_PACKET_TYPE (0x04)
    #[cfg_attr(kani, kani::ensures(|x: &Result<Self, DecodeError>|
        x.is_ok() == ((bytes.len() >= PreLoginHeader::LENGTH) &&
        ((bytes[0] == SERVER_PACKET_TYPE) || (bytes[0] == ClientMessageType::PreLogin))))
    )]
    /// Creates a new `PreLoginSpan` for parsing the pre-login stream.
    /// Err is returned if:
    /// - `bytes` length is less than PreLoginHeader::LENGTH
    /// - The first byte is not `PreLogin` or `PreLoginAck`
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < PreLoginHeader::LENGTH {
            #[cfg(not(kani))]
            return Err(DecodeError::InvalidLength(format!("PreLoginSpan::new() bytes.len()={} < PreLoginHeader::LENGTH={}", bytes.len(), PreLoginHeader::LENGTH)));
            #[cfg(kani)]
            return Err(DecodeError::KaniErrorStub)
        }
        if (bytes[0] != SERVER_PACKET_TYPE) &&
            (bytes[0] != ClientMessageType::PreLogin) {
            #[cfg(not(kani))]
            return Err(DecodeError::InvalidPacketType(format!("PreLoginSpan::new() unexpected packet type: 0x{:02x}", bytes[0])));
            #[cfg(kani)]
            return Err(DecodeError::KaniErrorStub)
        }
        Ok(PreLoginSpan {
            bytes,
            ..Default::default()
        })
    }
    #[inline(always)]
    #[cfg_attr(kani, kani::requires(self.bytes.len() >= PreLoginHeader::LENGTH))]
    /// Parses the TDS stream header.
    pub fn header(&self) -> PreLoginHeader {
        PreLoginHeader::from_bytes(&self.bytes[..8].try_into().unwrap_or([0u8;8]))
    }

    #[inline(always)]
    #[cfg_attr(kani, kani::requires(self.bytes.len() >= PreLoginHeader::LENGTH))]
    /// Performs a byte slice from the start of the pre-login options section.
    pub fn options(&self) -> PreLoginOptionsSpan<'a> {
        PreLoginOptionsSpan { bytes: &self.bytes[PreLoginHeader::LENGTH..], terminator: false }
    }
}

/// Zero-copy byte slice of the PreLogin options.
///
/// This struct implements an iterator that parses PreLogin options from a byte slice.
/// Each option is either 5 bytes (non-terminator) or 1 byte (terminator 0xFF).
///
/// ## Fields
/// - `bytes`: bytes for every pre-login option.
/// - `terminator`: Flag indicating whether the terminator option (0xFF) has been encountered
#[derive(Debug, Clone, Copy)]
pub struct PreLoginOptionsSpan<'a> {
    bytes: &'a [u8],
    terminator: bool,
}

impl<'a> Iterator for PreLoginOptionsSpan<'a> {
    type Item = PreLoginOptionSpan<'a>;

    // Post: if Some, then PLOptionType::Terminator (0xff) is 1 byte, otherwise, 5 bytes
    #[cfg_attr(kani, kani::ensures(|x|
        match x {
            Some(span) => {
                (span.bytes.len() == PreLoginPacket::TERMINATOR_OPT_LENGTH &&
                    span.bytes[0] == PLOptionType::Terminator) ||
                (span.bytes.len() == PreLoginPacket::OPT_LENGTH &&
                    span.bytes[0] != (PLOptionType::Terminator))
            },
            None => true,
        }
    ))]
    fn next(&mut self) -> Option<Self::Item> {
        // `PLOptionType::Terminator` is always the last option.
        if self.terminator {
            return None;
        }
        // Non-terminator option
        if self.bytes.len() >= PreLoginPacket::OPT_LENGTH
            && self.bytes[0] != (PLOptionType::Terminator)
        {
            let bytes = &self.bytes[..PreLoginPacket::OPT_LENGTH];
            self.bytes = &self.bytes[PreLoginPacket::OPT_LENGTH..];
            return Some(PreLoginOptionSpan { bytes });
        }
        // Terminator option. This is the last option.
        if self.bytes.len() >= PreLoginPacket::TERMINATOR_OPT_LENGTH
            && self.bytes[0] == PLOptionType::Terminator
        {
            let bytes = &self.bytes[..PreLoginPacket::TERMINATOR_OPT_LENGTH];
            self.bytes = &self.bytes[PreLoginPacket::TERMINATOR_OPT_LENGTH..];
            // Set self.terminator to true to disable next().
            self.terminator = true;
            return Some(PreLoginOptionSpan { bytes });
        }
        None
    }
    // fn payload(&mut self) -> &'a [u8] {
    //     if self.terminator {
    //         return self.bytes;
    //     }
    // }
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct PreLoginActivityID<'a> {
    _guid_activity_id: &'a [u8], // 16bytes
    _activity_sequence: u32,     // ulong
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromIntoFormat)]
// Pre-login option types and associated byte ID.
pub enum PLOptionType {
    Version = 0x00,    // BL_VERSION US_SUBBUILD
    Encryption = 0x01, // B_FENCRYPTION
    InstOpt = 0x02,    // B_INSTVALIDITY
    ThreadId = 0x03,   // UL_THREADID
    Mars = 0x04,       // B_MARS where 0x00 = Off, Ox01 = On
    #[cfg(feature = "tds7.4")]
    TraceId = 0x05, // GUID_CONNID ACTIVITYID
    #[cfg(feature = "tds7.4")]
    FedAuthRequired = 0x06, // B_FEDAUTHREQUIRED
    NonceOpt = 0x07,   // NONCE
    Terminator = 0xff,
}

impl PLOptionType {
    pub const PL_OPTION_MARS_OFF: bool = false;
    pub const PL_OPTION_MARS_ON: bool = true;
}

#[rustfmt::skip]
/// Parser for pre-login option.
impl<'a> PreLoginOptionSpan<'a> {
    #[cfg(not(feature = "unsafe"))]
    // Post: Some if bytes[0] is a valid PLOptionType, None otherwise.
    #[cfg_attr(kani, kani::ensures(|x: &Option<PLOptionType>|
        x.is_some() == (self.bytes[0] <= 0x07 || self.bytes[0] == 0xff)
    ))]
    pub fn option_token(&self) -> Option<PLOptionType> {
        PLOptionType::try_from(self.bytes[0]).ok()
    }

    /// # Safety
    #[cfg(feature = "unsafe")]
    #[inline(always)]
    // Post: Some if bytes[0] is a valid PLOptionType, None otherwise.
    #[cfg_attr(kani, kani::ensures(|x: &Option<PLOptionType>|
        x.is_some() == (self.bytes[0] <= 0x07 || self.bytes[0] == 0xff)
    ))]
    pub fn option_token(&self) -> Option<PLOptionType> {
        let byte = self.bytes[0];
        if matches!(byte, 0x00..=0x07 | 0xff) {
            return Some(unsafe { core::mem::transmute::<u8, PLOptionType>(byte)})
        }
        None
    }
    #[cfg(not(feature = "unsafe"))]
    #[inline(always)]
    // Post: if Some(x) is returned, then x is BE of bytes[1..2] of self.bytes
    #[cfg_attr(kani, kani::ensures(
        |x: &Option<u16>|
        x.map_or(true, |v| v == u16::from_be_bytes([self.bytes[1], self.bytes[2]]))
    ))]
    pub fn offset(&self) -> Option<u16> {
        if self.bytes.len() >= PreLoginPacket::OPT_LENGTH && self.bytes[0] != PLOptionType::Terminator {
            return Some(r_u16_be(self.bytes, 1))
        }
        None
    }
    /// # Safety
    #[cfg(feature = "unsafe")]
    #[cfg_attr(kani, kani::requires(self.bytes.len() == 5))]
    #[cfg_attr(kani, kani::requires(self.bytes[0] != (PLOptionType::Terminator)))]
    #[cfg_attr(kani, kani::ensures(
        |x: &Option<u16>|
        *x == Some(u16::from_be_bytes([self.bytes[1], self.bytes[2]]))
    ))]
    #[inline(always)]
    pub unsafe fn offset(&self) -> Option<u16> {
        Some(r_u16_be(self.bytes, 1))
    }

    #[cfg(not(feature = "unsafe"))]
    #[inline(always)]
    // Post: if Some(x) is returned, then x is BE of bytes[3..4] of self.bytes
    #[cfg_attr(kani, kani::ensures(
        |x: &Option<u16>|
        x.map_or(true, |v| v == u16::from_be_bytes([self.bytes[3], self.bytes[4]]))
    ))]
    pub fn option_length(&self) -> Option<u16> {
        if self.bytes.len() >= PreLoginPacket::OPT_LENGTH && self.bytes[0] != PLOptionType::Terminator {
            return Some(r_u16_be(self.bytes, 3))
        }
        None
    }
    /// # Safety
    #[cfg(feature = "unsafe")]
    #[cfg_attr(kani, kani::requires(self.bytes.len() == PreLoginPacket::OPT_LENGTH))]
    #[cfg_attr(kani, kani::requires(self.bytes[0] != (PLOptionType::Terminator)))]
    #[cfg_attr(kani, kani::ensures(
        |x: &Option<u16>|
        *x == Some(u16::from_be_bytes([self.bytes[3], self.bytes[4]]))
    ))]
    #[inline(always)]
    pub unsafe fn option_length(&self) -> Option<u16> {
        Some(r_u16_be(self.bytes, 3))
    }
}

#[cfg(not(feature = "tds8.0"))]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
#[repr(u8)]
pub enum PreLoginEncryptionOptions {
    Off = 0x00,
    On = 0x01,
    NotSupported = 0x02,
    Required = 0x03,
    Ext = 0x20,
    CertificateOff = 0x80,
    CertificateOn = 0x81,
    CertificateRequired = 0x83,
}

#[cfg(kani)]
#[kani::proof]
fn proof_non_terminator_opt() {
    let bytes: [u8; 13] = kani::any();
    kani::assume(bytes[0] == ClientMessageType::PreLogin);

    let span = PreLoginSpan::new(&bytes).unwrap();
    let mut iter = span.options();

    kani::assume(iter.bytes.len() >= PreLoginPacket::OPT_LENGTH && iter.bytes[0] <= 0x07);
    let opt = iter.next();

    assert!(opt.is_some());
    assert_eq!(opt.unwrap().bytes.len(), PreLoginPacket::OPT_LENGTH);
    assert!(iter.terminator == false);
}

#[cfg(kani)]
#[kani::proof]
fn proof_terminator_opt() {
    let bytes: [u8; 9] = kani::any();
    kani::assume(bytes[0] == ClientMessageType::PreLogin);

    let span = PreLoginSpan::new(&bytes).unwrap();
    let mut iter = span.options();

    kani::assume(iter.bytes.len() >= 1 && iter.bytes[0] == PLOptionType::Terminator);
    let opt = iter.next();

    assert!(opt.is_some());
    assert_eq!(opt.unwrap().bytes.len(), 1);
    assert!(iter.terminator == true);
}

#[cfg(kani)]
#[kani::proof]
fn proof_terminator_is_true() {
    let bytes: [u8; 14] = kani::any();
    kani::assume(bytes[0] == ClientMessageType::PreLogin);

    let span = PreLoginSpan::new(&bytes).unwrap();
    let mut iter = span.options();

    kani::assume(iter.bytes.len() >= 1 && iter.bytes[0] == PLOptionType::Terminator);
    let opt1 = iter.next();
    assert!(opt1.is_some());
    assert!(iter.terminator == true);

    let opt2 = iter.next();
    assert!(opt2.is_none());
}

#[cfg(kani)]
#[kani::proof]
fn proof_out_of_bounds_no_panic() {
    let bytes: [u8; PreLoginHeader::LENGTH - 1] = kani::any();
    let span = PreLoginSpan::new(&bytes);
    assert!(span.is_err());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Invalid pre-login request send from the client to the server
    #[test]
    pub fn test_header_is_none() {
        let bytes: [u8; 8] = [0x11, 0x01, 0x00, 0x2f, 0x00, 0x00, 0x01, 0x00];
        let span = PreLoginSpan::new(&bytes);
        assert!(span.is_err())
    }

    // Valid pre-login request send from the client to the server
    // Packet Header -
    // Packet Header Type = 12
    // Status = 01
    // Length = 00 2F
    // SPID = 00 00
    // PacketID = 01
    // Window = 00

    #[test]
    pub fn test_header_is_some() {
        let bytes: [u8; 8] = [
            ClientMessageType::PreLogin as u8,
            0x01,
            0x00,
            0x2f,
            0x00,
            0x00,
            0x01,
            0x00,
        ];
        let span = PreLoginSpan::new(&bytes);
        assert!(span.is_ok());
        let header = span.unwrap().header();
        assert_eq!(header.status, 0x01);
        assert_eq!(header.length, 0x2f);
        assert_eq!(header.spid, 0x00);
        assert_eq!(header.packet_id, 0x01);
        assert_eq!(header.window, 0x00);
    }

    #[cfg(feature = "unsafe")]
    #[test]
    pub fn test_options_iter() {
        let bytes: [u8; 47] = [
            ClientMessageType::PreLogin as u8,
            0x01,
            0x00,
            0x2F,
            0x00,
            0x00,
            0x01,
            0x00,
            0x00,
            0x00,
            0x1A,
            0x00,
            0x06,
            0x01,
            0x00,
            0x20,
            0x00,
            0x01,
            0x02,
            0x00,
            0x21,
            0x00,
            0x01,
            0x03,
            0x00,
            0x22,
            0x00,
            0x04,
            0x04,
            0x00,
            0x26,
            0x00,
            0x01,
            0xFF,
            0x09,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x01,
            0x00,
            0xB8,
            0x0D,
            0x00,
            0x00,
            0x01,
        ];
        let span = PreLoginSpan::new(&bytes);
        assert!(span.is_ok());
        // let header = span.unwrap().header();
        let span = span.unwrap();
        let mut iter = span.options();
        unsafe {
            let option = iter.next();
            assert!(option.is_some());
            // 0x00, 0x00, 0x1A, 0x00, 0x06,
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::Version));
            assert_eq!(option.offset(), Some(0x1a));
            assert_eq!(option.option_length(), Some(0x06));

            let option = iter.next();
            assert!(option.is_some());
            // 0x01, 0x00, 0x20, 0x00, 0x01
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::Encryption));
            assert_eq!(option.offset(), Some(0x20));
            assert_eq!(option.option_length(), Some(0x01));

            let option = iter.next();
            assert!(option.is_some());
            // 0x02, 0x00, 0x21, 0x00, 0x01,
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::InstOpt));
            assert_eq!(option.offset(), Some(0x21));
            assert_eq!(option.option_length(), Some(0x01));

            let option = iter.next();
            assert!(option.is_some());
            // 0x03, 0x00, 0x22, 0x00, 0x04,
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::ThreadId));
            assert_eq!(option.offset(), Some(0x22));
            assert_eq!(option.option_length(), Some(0x04));

            let option = iter.next();
            assert!(option.is_some());
            //  0x04, 0x00, 0x26, 0x00, 0x01
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::Mars));
            assert_eq!(option.offset(), Some(0x26));
            assert_eq!(option.option_length(), Some(0x01));

            let option = iter.next();
            assert!(option.is_some());
            // 0xFF
            let option = option.unwrap();
            assert_eq!(option.option_token(), Some(PLOptionType::Terminator));
            assert_eq!(option.offset(), None);
            assert_eq!(option.option_length(), None);
        }
    }
}
