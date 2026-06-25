#![allow(unused)]
use crate::tds::prelude::*;

#[cfg(kani)]
extern crate kani;

tds_packet_header!(Login7Header, ClientMessageType::TDS7Login);

#[cfg(feature = "tds7.4")]
span!(FeatureAckOptSpan, FeatureAckOptSpanIter);

/// 2.2.6.4 Login 7
#[derive(Debug, Clone, Default, Builder)]
#[builder(no_std, setter(strip_option), default)]
pub struct Login7Packet {
    pub(crate) length: u32,          // dword
    pub(crate) tds_version: u32,     // dword
    pub(crate) packet_size: u32,     // dword
    pub(crate) client_prog_ver: u32, // dword
    pub(crate) client_pid: u32,      // dword
    pub(crate) connection_id: u32,   // dword

    pub(crate) option_flag1: OptionFlag1,
    pub(crate) option_flag2: OptionFlag2,

    pub(crate) type_flags: TypeFlag,

    #[cfg(feature = "tds7.2")]
    pub(crate) option_flag3: OptionFlag3,
    pub(crate) client_time_zone: u32, // long

    #[deprecated]
    pub(crate) client_lc_id: u32, // ushort

    pub(crate) host_name: Option<String>,
    pub(crate) user_name: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) app_name: Option<String>,
    pub(crate) server_name: Option<String>,
    #[cfg(feature = "tds7.4")]
    extension: Option<String>,
    pub(crate) clt_int_name: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) client_id: Option<[u8; 6]>,
    pub(crate) sspi: Option<String>,
    pub(crate) atch_db_file: Option<String>,
    #[cfg(feature = "tds7.2")]
    pub(crate) change_password: Option<String>,
    #[cfg(feature = "tds7.2")]
    cb_sspi_long: u32, // dword

    data: Vec<u8>,
    #[cfg(feature = "tds7.4")]
    pub(crate) feature_ext: Vec<FeatureOption>,
}

impl Login7Packet {
    pub const PWD_XOR_MASK: u8 = 0xa5;
}

impl core::fmt::Display for Login7Packet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Login7 \nlength:\t\t{}\n", self.length,)
    }
}

impl Login7Header {
    // Length +
    // TDSVersion +
    // PacketSize +
    // ClientProgVer +
    // ClientPID +
    // ConnectionID = 6 * 4 = 24
    // OptionFlags1 = 1
    // OptionFlags2 = 1
    // TypeFlags = 1
    // OptionFlag3 = 1
    // ClientTimeZone = 4
    // ClientLCID = 4
    pub const VARIABLE_LENGTH_TABLE_OFFSET: usize = 36 + 8;
    #[cfg(feature = "tds7.2")]
    pub const FIXED_DATA_OFFSET: usize = 102;
    #[cfg(not(feature = "tds7.2"))]
    pub const FIXED_DATA_OFFSET: usize = 94;
}
// The const idents are mapped from the TDS specification (ref: 2.2.6.4 LOGIN7)
// into this enum in descending int/bit order. Bits are converted to boolean using
// standard convention. However, NOTE that flags may be inverted semantically.
#[rustfmt::skip]
impl OptionFlag1 {
    pub fn new(
        f_byte_order: bool,
        f_char: bool,
        f_float: u8,
        f_dump_load: bool,
        f_use_db: bool,
        f_database: bool,
        f_set_lang: bool
    ) -> Self {
            let option_flag1 = (f_byte_order as u8)
                | (f_char as u8) << 1
                | f_float << 2
                | (f_dump_load as u8) << 4
                | (f_use_db as u8) << 5
                | (f_database as u8) << 6
                | (f_set_lang as u8) << 7;

            Self(option_flag1)
    }
    pub const F_BYTE_ORDER_X86: bool = false;
    pub const F_BYTE_ORDER_68000: bool = true;
    #[inline(always)]
    pub fn f_byte_order(&self) -> bool { self.0 & 0x01 != 0 } // bit
    pub const F_CHAR_CHARSET_ASCII: bool = false;
    pub const F_CHAR_CHARSET_EBCDIC: bool = true;
    #[inline(always)]
    pub fn f_char(&self) -> bool { self.0 & 0x02 != 0 } // bit
    pub const F_FLOAT_IEEE_754: u8 = 0x00;
    pub const F_FLOAT_VAX: u8 = 0x01;
    pub const F_FLOAT_ND5000: u8 = 0x02;
    #[inline(always)]
    pub fn f_float(&self) -> u8 { (self.0 & 0x0c) >> 2 } // 2 bit
    pub const F_DUMPLOAD_ON : bool = false;
    pub const F_DUMPLOAD_OFF : bool = true;
    #[inline(always)]
    pub fn f_dump_load(&self) -> bool { self.0 & 0x10 != 0 } // bit
    pub const F_USE_DB_OFF : bool = false;
    pub const F_USE_DB_ON : bool = true;
    #[inline(always)]
    pub fn f_use_db(&self) -> bool { self.0 & 0x20 != 0 } // bit
    pub const F_DATABASE_WARN : bool = false;
    pub const F_DATABASE_FATAL : bool = true;
    #[inline(always)]
    pub fn f_database(&self) -> bool { self.0 & 0x40 != 0 } // bit
    pub const F_SET_LANG_OFF : bool = false;
    pub const F_SET_LANG_ON : bool = true;
    #[inline(always)]
    pub fn f_set_lang(&self) -> bool { self.0 & 0x80 != 0 } // bit
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OptionFlag1(pub u8);

#[rustfmt::skip]
impl OptionFlag2 {
    pub fn new(
        f_language: bool,
        f_odbc: bool,
        f_trans_boundary: bool,
        f_cache_connect: bool,
        f_user_type: u8,
        f_int_security: bool,
    ) -> Self {
            let option_flag2 = (f_language as u8)
                | (f_odbc as u8) << 1
                | (f_trans_boundary as u8) << 2
                | (f_cache_connect as u8) << 3
                | f_user_type << 4
                | (f_int_security as u8) << 7;

            Self(option_flag2)
    }
    pub const F_INIT_LANG_WARN: bool = false;
    pub const F_INIT_LANG_FATAL: bool = true;
    #[inline(always)]
    pub fn f_language(&self) -> bool { self.0 & 0x01 != 0} // bit
    pub const F_ODBC_OFF: bool = false;
    pub const F_ODBC_ON: bool = true;
    #[inline(always)]
    pub fn f_odbc(&self) -> bool { self.0 & 0x02 != 0} // bit
    #[inline(always)]
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    pub fn f_trans_boundary(&self) -> bool { self.0 & 0x04 != 0} // bit
    #[inline(always)]
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    pub fn f_cache_connect(&self) -> bool { self.0 & 0x08 != 0} // bit
    pub const F_USER_TYPE_NORMAL: u8 = 0x00;
    pub const F_USER_TYPE_SERVER: u8 = 0x01;
    pub const F_USER_TYPE_REMUSER: u8 = 0x02;
    pub const F_USER_TYPE_SQLREPL: u8 = 0x03;
    #[inline(always)]
    pub fn f_user_type(&self) -> u8 { (self.0 & 0x70) >> 4 } // 3bit
    pub const F_INT_SECURITY_OFF: bool = false;
    pub const F_INT_SECURITY_ON: bool = true;
    #[inline(always)]
    pub fn f_int_security(&self) -> bool { self.0 & 0x80 != 0} // bit
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OptionFlag2(pub u8);

impl TypeFlag {
    pub fn new(f_sql_type: u8, f_oledb: bool, f_read_only_intent: bool) -> Self {
        let type_flag = f_sql_type | (f_oledb as u8) << 4 | (f_read_only_intent as u8) << 5;
        Self(type_flag)
    }
    pub const F_SQL_DFLT: bool = false;
    pub const F_SQL_TSQL: bool = true;
    #[inline(always)]
    pub fn f_sql_type(&self) -> u8 {
        self.0 & 0x0f
    } // 4bit
    #[cfg(feature = "tds7.2")]
    pub const F_OLEDB_OFF: bool = false;
    #[cfg(feature = "tds7.2")]
    pub const F_OLEDB_ON: bool = true;
    #[cfg(feature = "tds7.2")]
    #[inline(always)]
    pub fn f_oledb(&self) -> bool {
        self.0 & 0x10 != 0
    } // bit
    #[cfg(feature = "tds7.4")]
    #[inline(always)]
    pub fn f_read_only_intent(&self) -> bool {
        self.0 & 0x20 != 0
    } // bit
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TypeFlag(pub u8);

#[cfg(feature = "tds7.2")]
impl OptionFlag3 {
    fn new(f_change_password: bool, f_user_instance: bool, f_send_yukon_binary_xml: bool,
    f_unknown_collation_handling: bool, f_extension: bool) -> Self {
        let options_flag3 =  (f_change_password as u8)
        | (f_user_instance as u8) << 1
        | (f_send_yukon_binary_xml as u8) << 2
        | (f_unknown_collation_handling as u8) << 3
        | (f_extension as u8) << 4;

        Self(options_flag3)
    }
    #[inline(always)]
    pub fn f_change_password(&self) -> bool {
        self.0 & 0x01 != 0
    } // bit
    #[inline(always)]
    pub fn f_user_instance(&self) -> bool {
        self.0 & 0x02 != 0
    } // bit
    #[inline(always)]
    pub fn f_send_yukon_binary_xml(&self) -> bool {
        self.0 & 0x04 != 0
    } // bit
    #[inline(always)]
    #[cfg(feature = "tds7.3")]
    pub fn f_unknown_collation_handling(&self) -> bool {
        self.0 & 0x08 != 0
    } // bit
    #[inline(always)]
    #[cfg(feature = "tds7.4")]
    pub fn f_extension(&self) -> bool {
        self.0 & 0x10 != 0
    } // bit
}

#[cfg(feature = "tds7.2")]
#[derive(Debug, Clone, Copy, Default)]
pub struct OptionFlag3(pub u8);

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone)]
pub enum FeatureOption {
    SessionRecovery(SessionRecoveryData),
    FedAuth(FedAuthKind),
    ColumnEncryption(ColumnEncryptionData),
    GlobalTransactions,                   // NO DATA
    AzureSQLSupport(AzureSQLSupportData), // newtype = u8
    DataClassification(DataClassificationData),
    UTF8Support(UTF8SupportData),     // newtype = u8
    AzureSQLDNSCaching,               // NO DATA
    JsonSupport(JsonSupportData),     // newtype = u8
    VectorSupport(VectorSupportData), // newtype = u8
    EnhancedRoutingSupport,           // NO DATA
    UserAgent,                        // us_varchar
    Terminator,
}

#[cfg(feature = "tds7.4")]
impl FeatureOption {
    pub const LENGTH: usize = 5;
}

#[cfg(feature = "tds7.4")]
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum FeatureExtType {
    SessionRecovery = 0x01,
    FedAuth = 0x02,
    ColumnEncryption = 0x04,
    GlobalTransactions = 0x05,
    AzureSQLSupport = 0x08,
    DataClassification = 0x09,
    UTF8Support = 0x0a,
    AzureSQLDNSCaching = 0x0b,
    JsonSupport = 0x0d,
    VectorSupport = 0x0e,
    EnhancedRoutingSupport = 0x0f,
    Terminator = 0xff,
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone)]
pub struct SessionRecoveryData {
    pub(crate) length: u32,                 //dword
    pub(crate) recovery_database: String,   //b_varchar
    pub(crate) recovery_collation: Collation, //bytelen [collation]
    pub(crate) recovery_language: String,
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone)]
pub enum FedAuthKind {
    #[cfg(feature = "fed-auth-adal")]
    Adal(FedAuthAdal),
    #[cfg(feature = "fed-auth-token")]
    SecurityToken(FedAuthSecurityToken),
}

#[cfg(all(feature = "tds7.4", feature = "fed-auth-adal"))]
#[derive(Debug, Clone)]
pub struct FedAuthAdal {
    pub(crate) echo: bool,
    pub(crate) workflow: FedAuthWorkflowType,
}

#[cfg(all(feature = "tds7.4", feature = "fed-auth-token"))]
#[derive(Debug, Clone)]
pub struct FedAuthSecurityToken {
    pub(crate) echo: bool,
    pub(crate) fed_auth_token: String,   // l_varchar
    pub(crate) nonce: [u8; 32],
    pub(crate) channel_binding_token: Vec<u8>,
    pub(crate) signature: [u8; 32],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromIntoFormat)]
pub enum FedAuthWorkflowType {
    UsernamePassword = 0x01,
    Integrated = 0x02,
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct ColumnEncryptionData {
    pub(crate) version: ColumnEncryptionVersion,
}

#[cfg(feature = "tds7.4")]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ColumnEncryptionVersion {
    WithoutEnclaveComputations = 0x01,
    EncryptedRequiresEnclaveComputations = 0x02,
    EncryptedRequiresEnclaveComputationsWithCaching = 0x03,
}

#[allow(unused)]
#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy, Default)]
pub struct AzureSQLSupportData(pub(crate) bool);

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct DataClassificationData {
    pub(crate) _version: DataClassificationVersion,
}

#[cfg(feature = "tds7.4")]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum DataClassificationVersion {
    NoSensitivityRankData = 1,
    SensitivityRankData = 2,
}

#[allow(unused)]
#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy, Default)]
pub struct UTF8SupportData(pub(crate) bool);

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct JsonSupportData {
    pub(crate) _version: u8,
}

#[cfg(feature = "tds7.4")]
#[derive(Debug, Clone, Copy)]
pub struct VectorSupportData {
    pub(crate) _version: u8,
}

#[cfg(feature = "tds7.4")]
impl<'a> FeatureExtAckSpan<'a> {
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 6 { return Err(DecodeError::InvalidData("".to_string())) }
        Ok(Self { bytes })
    }
    pub fn token_type(&self) -> u8 {
        self.bytes[0]
    }
    pub fn feature_ack_opt(&self) -> FeatureAckOptSpanIter<'a> {
        FeatureAckOptSpanIter { bytes: &self.bytes[1..] }
    }
}

impl<'a> Iterator for FeatureAckOptSpanIter<'a> {
    type Item = FeatureAckOptSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if *self.bytes.first()? == 0xff { return None }
        let length = u32::from_le_bytes(self.bytes.get(1..5)?.try_into().ok()?) as usize;
        let opt = FeatureAckOptSpan { bytes: self.bytes.get(..5 + length)? };
        self.bytes = &self.bytes[5 + length..];
        Some(opt)
    }
}

#[cfg(feature = "tds7.4")]
impl<'a> FeatureAckOptSpan<'a> {
    pub fn ty(&self) -> Option<FeatureExtType> {
        FeatureExtType::from_u8(self.feature_id())
    }
    pub fn feature_id(&self) -> u8 {
        self.bytes[0]
    }
    pub fn feature_ack_data_len(&self) -> u32 {
        (self.bytes.len() - 5) as u32
    }
    pub fn feature_ack_data(&self) -> &'a [u8] { 
        &self.bytes[5..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // option_flag1
    #[test]
    fn test_option_flag1_f_byte_order() {
        assert_eq!(OptionFlag1::new(OptionFlag1::F_BYTE_ORDER_X86, false, 0, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(OptionFlag1::F_BYTE_ORDER_68000, false, 0, false, false, false, false).0, 0x01); // 1

    }

    #[test]
    fn test_option_flag1_f_char() {
        assert_eq!(OptionFlag1::new(false, OptionFlag1::F_CHAR_CHARSET_ASCII, 0, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, OptionFlag1::F_CHAR_CHARSET_EBCDIC, 0, false, false, false, false).0, 0x02); // 2

    }

    #[test]
    fn test_option_flag1_f_float() {
        assert_eq!(OptionFlag1::new(false, false, OptionFlag1::F_FLOAT_IEEE_754, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, false, OptionFlag1::F_FLOAT_VAX, false, false, false, false).0, 0x04); // 4
        assert_eq!(OptionFlag1::new(false, false, OptionFlag1::F_FLOAT_ND5000, false, false, false, false).0, 0x08); // 8
    }

    #[test]
    fn test_option_flag1_f_dump_load() {
        assert_eq!(OptionFlag1::new(false, false, 0, OptionFlag1::F_DUMPLOAD_ON, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, false, 0, OptionFlag1::F_DUMPLOAD_OFF, false, false, false).0, 0x10); // 16

    }

    #[test]
    fn test_option_flag1_f_use_db() {
        assert_eq!(OptionFlag1::new(false, false, 0, false, OptionFlag1::F_USE_DB_OFF, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, false, 0, false, OptionFlag1::F_USE_DB_ON, false, false).0, 0x20); // 32

    }

    #[test]
    fn test_option_flag1_f_database() {
        assert_eq!(OptionFlag1::new(false, false, 0, false, false, OptionFlag1::F_DATABASE_WARN, false).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, false, 0, false, false, OptionFlag1::F_DATABASE_FATAL, false).0, 0x40); // 64

    }

    #[test]
    fn test_option_flag1_f_set_lang() {
        assert_eq!(OptionFlag1::new(false, false, 0, false, false, false, OptionFlag1::F_SET_LANG_OFF).0, 0u8); // 0
        assert_eq!(OptionFlag1::new(false, false, 0, false, false, false, OptionFlag1::F_SET_LANG_ON).0, 0x80); // 128
    }

    /// option_flag2
    #[test]
    fn test_option_flag2_f_language_warn() {
        let f_lang_warn = OptionFlag2::new(OptionFlag2::F_INIT_LANG_WARN, false, false, false, 0u8, false);
        assert_eq!(f_lang_warn.0, 0u8); // 0
        let f_lang_err = OptionFlag2::new(OptionFlag2::F_INIT_LANG_FATAL, false, false, false, 0u8, false);
        assert_eq!(f_lang_err.0, 1u8); // 1

    }

    #[test]
    fn test_option_flag2_f_odbc() {
        let f_odbc_on = OptionFlag2::new(false, OptionFlag2::F_ODBC_ON, false, false, 0u8, false);
        assert_eq!(f_odbc_on.0, 0x02); // 2
        let f_odbc_off = OptionFlag2::new(false, OptionFlag2::F_ODBC_OFF, false, false, 0u8, false);
        assert_eq!(f_odbc_off.0, 0x00); // 0
    }

    #[test]
    fn test_option_flag2_f_trans_boundary() {
        let option_flags2 = OptionFlag2::new(false, false, true, false, 0u8, false);
        assert_eq!(option_flags2.0, 0x04); // 4
    }

    #[test]
    fn test_option_flag2_f_cache_connect() {
        let option_flags2 = OptionFlag2::new(false, false, false, true, 0u8, false);
        assert_eq!(option_flags2.0, 0x08); // 8
    }

    #[test]
    fn test_option_flag2_f_user_ty() {
        let f_user_ty_norm = OptionFlag2::new(false, false, false, false, OptionFlag2::F_USER_TYPE_NORMAL, false);
        assert_eq!(f_user_ty_norm.0, 0x00); // 0
        let f_user_ty_serv = OptionFlag2::new(false, false, false, false, OptionFlag2::F_USER_TYPE_SERVER, false);
        assert_eq!(f_user_ty_serv.0, 0x10); // 16
        let f_user_ty_rem: OptionFlag2 = OptionFlag2::new(false, false, false, false, OptionFlag2::F_USER_TYPE_REMUSER, false);
        assert_eq!(f_user_ty_rem.0, 0x20); // 32
        let f_user_ty_sql_repl: OptionFlag2 = OptionFlag2::new(false, false, false, false, OptionFlag2::F_USER_TYPE_SQLREPL, false);
        assert_eq!(f_user_ty_sql_repl.0, 0x30); // 48
    }

    #[test]
    fn test_option_flag2_f_int_security() {
        let f_int_security_on = OptionFlag2::new(false, false, false, false, 0u8, OptionFlag2::F_INT_SECURITY_ON);
        assert_eq!(f_int_security_on.0, 0x80); // 128
        let f_int_security_off = OptionFlag2::new(false, false, false, false, 0u8, OptionFlag2::F_INT_SECURITY_OFF);
        assert_eq!(f_int_security_off.0, 0x00); // 0
    }
    /// type_flag
    #[test]
    fn test_type_flag_f_sql_type() {
        let type_flag = TypeFlag::new(TypeFlag::F_SQL_DFLT as u8, false,  false);
        assert_eq!(type_flag.0, 0x00); // 0
        let type_flag = TypeFlag::new(TypeFlag::F_SQL_TSQL as u8, false,  false);
        assert_eq!(type_flag.0, 0x01); // 1
    }
    
    #[cfg(feature = "tds7.2")]
    #[test]
    fn test_type_flag_f_oledb() {
        let type_flag = TypeFlag::new(0u8, TypeFlag::F_OLEDB_ON,  false);
        assert_eq!(type_flag.0, 0x10);
        let type_flag = TypeFlag::new(0u8, TypeFlag::F_OLEDB_OFF,  false);
        assert_eq!(type_flag.0, 0u8);
    }
    
    #[test]
    fn test_type_flag_f_read_only_intent() {
        let type_flag = TypeFlag::new(0u8, false, true);
        assert_eq!(type_flag.0, 0x20);
        let type_flag = TypeFlag::new(0u8, false, false);
        assert_eq!(type_flag.0, 0u8);
    }
    //option_flag3
    
    #[cfg(feature = "tds7.2")]
    #[test]
     fn test_option_flag3_f_change_password() {
        assert_eq!(OptionFlag3::new(false, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag3::new(true, false, false, false, false).0, 0x01); // 1
    }

    #[cfg(feature = "tds7.2")]
    #[test]
    fn test_option_flag3_f_user_instance() {
        assert_eq!(OptionFlag3::new(false, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag3::new(false, true, false, false, false).0, 0x02); // 2
    }

    #[cfg(feature = "tds7.2")]
    #[test]
    fn test_option_flag3_f_send_yukon_binary_xml() {
        assert_eq!(OptionFlag3::new(false, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag3::new(false, false, true, false, false).0, 0x04); // 4

    }

    #[cfg(feature = "tds7.2")]
    #[test]
    fn test_option_flag3_f_unknown_collation_handling() {
        assert_eq!(OptionFlag3::new(false, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag3::new(false, false, false, true, false).0, 0x08); // 8
    }

    #[cfg(feature = "tds7.2")]
    #[test]
    fn test_option_flag3_f_extension() {
        assert_eq!(OptionFlag3::new(false, false, false, false, false).0, 0u8); // 0
        assert_eq!(OptionFlag3::new(false, false, false, false, true).0, 0x10); // 16
    }
}