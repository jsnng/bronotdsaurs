use crate::tds::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromIntoFormat)]
pub enum EnvChangeType {
    Database = 0x01,
    Language = 0x02,
    #[cfg_attr(feature = "tds7.1", deprecated)]
    CharacterSet = 0x03,
    PacketSize = 0x04,
    #[cfg_attr(feature = "tds7.1", deprecated)]
    UnicodeDataSortingLocalID = 0x05,
    #[cfg_attr(feature = "tds7.1", deprecated)]
    UnicodeDataSortingComparisonFlags = 0x06,
    SQLCollation = 0x07,
    #[cfg(feature = "tds7.2")]
    BeginTransaction = 0x08,
    #[cfg(feature = "tds7.2")]
    CommitTransaction = 0x09,
    #[cfg(feature = "tds7.2")]
    RollbackTransaction = 0xa,
    #[cfg(feature = "tds7.2")]
    EnlistDTCTransaction = 0xb,
    #[cfg(feature = "tds7.2")]
    DefectTransaction = 0xc,
    #[cfg(feature = "tds7.2")]
    RealTimeLogShipping = 0xd,
    #[cfg(feature = "tds7.2")]
    PromoteTransaction = 0xf,
    #[cfg(feature = "tds7.2")]
    #[deprecated]
    TransactionManagerAddress = 0x10,
    #[cfg(feature = "tds7.2")]
    TransactionEnded = 0x11,
    #[cfg(feature = "tds7.2")]
    RESETCONNECTIONCompletionAck = 0x12,
    #[cfg(feature = "tds7.2")]
    SendUserToClientRequest = 0x13,
    #[cfg(feature = "tds7.4")]
    SendRoutingInformation = 0x14,
    #[cfg(feature = "tds7.4")]
    SendEnhancedRoutingInformation = 0x15,
}

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum EnvValueData {
    BVarChar { new: String, old: String },
    BVarBytes { new: Vec<u8>, old: Vec<u8> },
    Routing {},
    EnhancedRouting {},
}

impl EnvValueData {
    pub fn new_value(&self) -> Option<&str> {
        match self {
            Self::BVarChar { new, .. } => Some(new.as_str()),
            _ => None,
        }
    }

    pub fn old_value(&self) -> Option<&str> {
        match self {
            Self::BVarChar { old, .. } => Some(old.as_str()),
            _ => None,
        }
    }

    pub fn new_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BVarBytes { new, .. } => Some(new.as_slice()),
            _ => None,
        }
    }

    pub fn old_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BVarBytes { old, .. } => Some(old.as_slice()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvChangeToken {
    pub ty: EnvChangeType,
    pub(crate) length: usize,
    pub env_value_data: EnvValueData,
}

impl<'a> EnvChangeSpan<'a> {
    pub const FIXED_SPAN_SIZE: usize = 4;

    // Post: Ok(s) ==> bytes.len() == s.length() + 3
    #[cfg_attr(kani, kani::ensures(|x: &Result<Self, DecodeError>|
        if let Ok(s) = x { s.length() as usize + 3 == bytes.len() } else { true }
    ))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::FIXED_SPAN_SIZE {
            return Err(DecodeError::InvalidLength(format!(
                "EnvChangeSpan::new() bytes.len()={} < FIXED_SPAN_SIZE={}",
                bytes.len(),
                Self::FIXED_SPAN_SIZE
            )));
        }
        let env_change = Self { bytes };
        // length gives the token body size only. bytes includes ty (u8) and length (u16).
        if env_change.length() as usize + 3 != bytes.len() {
            return Err(DecodeError::InvalidLength(format!(
                "EnvChangeSpan::new() length+3={} != bytes.len()={}",
                env_change.length() as usize + 3,
                bytes.len()
            )));
        }
        Ok(env_change)
    }
}

impl<'a> EnvChangeSpan<'a> {
    // // Post:
    // #[cfg_attr(kani, kani::ensures(|x: &Option<EnvChangeType>|
    //     match x {
    //         Some(ty) => *ty as u8 == self.bytes[3],
    //         None => !matches(self.bytes[3], 0x01..=0x0d | 0x0f..0x15),
    //     }
    // ))]
    pub fn ty(&self) -> Option<EnvChangeType> {
        EnvChangeType::from_u8(self.bytes[3])
    }

    #[inline(always)]
    // length gives the token body size only.
    pub fn length(&self) -> u16 {
        let cursor: usize = 1;
        r_u16_le(self.bytes, cursor)
    }

    #[inline(always)]
    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn env_value_data(&self) -> &'a [u8] {
        if let Ok(cch) = self.cch_new_value() {
            let ib = self.ib_env_value_data();
            return &self.bytes[ib..ib + cch];
        }
        &[]
    }

    #[inline(always)]
    const fn ib_env_value_data(&self) -> usize {
        Self::FIXED_SPAN_SIZE
    }

    /// Read the length prefix at `ib` and scale it by the type's multiplier.
    /// B_VARCHAR types store char counts (×2 for UTF-16), B_VARBYTE types store raw byte counts.
    #[inline(always)]
    fn cch_at(&self, ib: usize) -> Result<usize, DecodeError> {
        let multiplier = match self.ty() {
            #[allow(deprecated)]
            Some(EnvChangeType::Database)
            | Some(EnvChangeType::Language)
            | Some(EnvChangeType::CharacterSet)
            | Some(EnvChangeType::PacketSize)
            | Some(EnvChangeType::UnicodeDataSortingLocalID)
            | Some(EnvChangeType::UnicodeDataSortingComparisonFlags) => 2,
            #[cfg(feature = "tds7.2")]
            Some(EnvChangeType::RealTimeLogShipping)
            | Some(EnvChangeType::SendUserToClientRequest) => 2,
            Some(EnvChangeType::SQLCollation) => 1,
            #[cfg(feature = "tds7.2")]
            Some(EnvChangeType::BeginTransaction)
            | Some(EnvChangeType::CommitTransaction)
            | Some(EnvChangeType::RollbackTransaction)
            | Some(EnvChangeType::EnlistDTCTransaction)
            | Some(EnvChangeType::DefectTransaction)
            | Some(EnvChangeType::TransactionEnded) => 1,
            #[cfg(feature = "tds7.2")]
            #[allow(deprecated)]
            Some(EnvChangeType::TransactionManagerAddress) => 1,
            #[cfg(feature = "tds7.2")]
            Some(EnvChangeType::RESETCONNECTIONCompletionAck) => return Ok(0),
            #[cfg(feature = "tds7.2")]
            Some(EnvChangeType::PromoteTransaction) => return Ok(r_u32_le(self.bytes, ib) as usize),
            #[cfg(feature = "tds7.4")]
            Some(EnvChangeType::SendRoutingInformation)
            | Some(EnvChangeType::SendEnhancedRoutingInformation) => return Ok(r_u16_le(self.bytes, ib) as usize),
            None => return Err(DecodeError::InvalidEnvChangeType(format!("unknown env change type byte: 0x{:02x}", self.bytes[3]))),
        };
        Ok(self.bytes[ib] as usize * multiplier)
    }

    #[inline(always)]
    pub fn ib_new_value(&self) -> Result<usize, DecodeError> {
        Ok(self.ib_env_value_data() + 1)
    }

    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn cch_new_value(&self) -> Result<usize, DecodeError> {
        self.cch_at(self.ib_env_value_data())
    }

    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn ib_old_value(&self) -> Result<usize, DecodeError> {
        Ok(self.ib_new_value()? + self.cch_new_value()? + 1)
    }

    // Post:
    #[cfg_attr(kani, kani::ensures(|_| true))]
    pub fn cch_old_value(&self) -> Result<usize, DecodeError> {
        self.cch_at(self.ib_old_value()? - 1)
    }
}
