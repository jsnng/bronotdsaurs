use crate::tds::prelude::*;
use derive_proc_macros::TryFromIntoFormat;

#[cfg(kani)]
extern crate kani;

/// 2.2.5.4 Data Type Definitions
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    ZeroLength(ZeroLengthDataType),
    Fixed(FixedLengthDataType),
    Variable(VariableLengthDataType),
}

impl DataType {
    pub const TYPE_CLASSIFICATION_LUT: [u8; 256] = {
        let mut x = [0u8; 256];
        x[ZeroLengthDataType::Null as usize] = 1;
        x[FixedLengthDataType::Int1 as usize] = 2;
        x[FixedLengthDataType::Bit as usize] = 2;
        x[FixedLengthDataType::Int2 as usize] = 2;
        x[FixedLengthDataType::Int4 as usize] = 2;
        x[FixedLengthDataType::DateTim4 as usize] = 2;
        x[FixedLengthDataType::Flt4 as usize] = 2;
        x[FixedLengthDataType::DateTime as usize] = 2;
        x[FixedLengthDataType::Flt8 as usize] = 2;
        #[cfg(feature = "legacy")]
        {
            x[FixedLengthDataType::Decimal as usize] = 2;
            x[FixedLengthDataType::Numeric as usize] = 2;
        }
        x[FixedLengthDataType::Money4 as usize] = 2;
        x[FixedLengthDataType::Money as usize] = 2;
        x[FixedLengthDataType::Int8 as usize] = 2;
        x[VariableLengthDataType::Guid as usize] = 3;
        x[VariableLengthDataType::IntN as usize] = 3;
        x[VariableLengthDataType::BitN as usize] = 3;
        x[VariableLengthDataType::DecimalN as usize] = 3;
        x[VariableLengthDataType::NumericN as usize] = 3;
        x[VariableLengthDataType::FltN as usize] = 3;
        x[VariableLengthDataType::MoneyN as usize] = 3;
        x[VariableLengthDataType::DateTimN as usize] = 3;
        #[cfg(feature = "tds7.3")]
        {
            x[VariableLengthDataType::DateN as usize] = 3;
            x[VariableLengthDataType::TimeN as usize] = 3;
            x[VariableLengthDataType::DateTime2N as usize] = 3;
            x[VariableLengthDataType::DateTimeOffsetN as usize] = 3;
        }
        x[VariableLengthDataType::BigVarBinary as usize] = 3;
        x[VariableLengthDataType::BigVarChar as usize] = 3;
        x[VariableLengthDataType::BigBinary as usize] = 3;
        x[VariableLengthDataType::BigChar as usize] = 3;
        x[VariableLengthDataType::NVarChar as usize] = 3;
        x[VariableLengthDataType::NChar as usize] = 3;
        x[VariableLengthDataType::Text as usize] = 3;
        x[VariableLengthDataType::Image as usize] = 3;
        x[VariableLengthDataType::NText as usize] = 3;
        #[cfg(feature = "tds7.2")]
        {
            x[VariableLengthDataType::SsVariant as usize] = 3;
        }
        x[VariableLengthDataType::Json as usize] = 3;
        x[VariableLengthDataType::Vector as usize] = 3;
        x
    };
}

impl TryFrom<u8> for DataType {
    type Error = DecodeError;

    fn try_from(val: u8) -> Result<Self, DecodeError> {
        match DataType::TYPE_CLASSIFICATION_LUT[val as usize] {
            1 => Ok(Self::ZeroLength(ZeroLengthDataType::from_u8(val).unwrap())),
            2 => Ok(Self::Fixed(FixedLengthDataType::from_u8(val).unwrap())),
            3 => Ok(Self::Variable(
                VariableLengthDataType::from_u8(val).unwrap(),
            )),
            _ => Err(DecodeError::InvalidField(format!(
                "DataType::try_from() unknown value: 0x{:02x}",
                val
            ))),
        }
    }
}

impl From<DataType> for u8 {
    fn from(val: DataType) -> Self {
        match val {
            DataType::ZeroLength(x) => x as u8,
            DataType::Fixed(x) => x as u8,
            DataType::Variable(x) => x as u8,
        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromIntoFormat)]
pub enum ZeroLengthDataType {
    Null = 0x1f,
}

impl ZeroLengthDataType {
    // zero-length data types have a length of... 0
    #[inline(always)]
    pub const fn size(&self) -> usize {
        0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromIntoFormat)]
pub enum FixedLengthDataType {
    Bit = 0x32,
    DateTim4 = 0x3a,
    DateTime = 0x3d,
    #[cfg(feature = "legacy")]
    #[deprecated]
    Decimal = 0x37,
    Flt4 = 0x3b,
    Flt8 = 0x3e,
    Int1 = 0x30,
    Int2 = 0x34,
    Int4 = 0x38,
    Int8 = 0x7f,
    Money = 0x3c,
    Money4 = 0x7a,
    #[cfg(feature = "legacy")]
    #[deprecated]
    Numeric = 0x3f,
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromIntoFormat)]
pub enum VariableLengthDataType {
    Guid = 0x24,
    IntN = 0x26,
    BitN = 0x68,
    DecimalN = 0x6a,
    NumericN = 0x6c,
    FltN = 0x6d,
    MoneyN = 0x6e,
    DateTimN = 0x6f,
    #[cfg(feature = "tds7.3")]
    DateN = 0x28,
    #[cfg(feature = "tds7.3")]
    TimeN = 0x29,
    #[cfg(feature = "tds7.3")]
    DateTime2N = 0x2a,
    #[cfg(feature = "tds7.3")]
    DateTimeOffsetN = 0x2b,
    #[cfg(feature = "legacy")]
    #[deprecated]
    Char = 0x2f,
    #[cfg(feature = "legacy")]
    #[deprecated]
    VarChar = 0x27,
    #[cfg(feature = "legacy")]
    #[deprecated]
    Binary = 0x2d,
    #[cfg(feature = "legacy")]
    #[deprecated]
    VarBinary = 0x25,

    BigVarBinary = 0xa5,
    BigVarChar = 0xa7,
    BigBinary = 0xad,
    BigChar = 0xaf,
    NVarChar = 0xe7,
    NChar = 0xef,
    #[cfg(feature = "tds7.2")]
    Xml = 0xf1,
    #[cfg(feature = "tds7.2")]
    Udt = 0xf0,
    Text = 0x23,
    Image = 0x22,
    NText = 0x63,
    #[cfg(feature = "tds7.2")]
    SsVariant = 0x62,
    Json = 0xf4,
    Vector = 0xf5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VectorDimensionType {
    BigBinary,
    BigVarBinary,
    BigVarChar,
}

#[cfg(feature = "tds7.4")]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithmType {
    Deterministic = 1,
    Randomized = 2,
}

#[cfg(feature = "tds7.4")]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum EncryptionAlgorithm {
    Custom = 0,
    AeadAes256CbcHmacSha512 = 1,
    AeadAes256CbcHmacSha256 = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct TypeInfoSpan<'a> {
    pub bytes: &'a [u8],
    pub dtype: DataType,
}

impl<'a> TypeInfoSpan<'a> {
    #[inline(always)]
    pub fn new(dtype: DataType, bytes: &'a [u8]) -> Self {
        Self { bytes, dtype }
    }
}

#[derive(Debug, Clone, Copy, Builder)]
#[builder(no_std, build_fn(validate = "Self::validate"))]
pub struct TypeInfo {
    pub(crate) dtype: DataType,
    pub(crate) dtype_max_len: Option<TypeInfoVarLen>,
    pub(crate) collation: Option<Collation>,
    pub(crate) precision: Option<u8>,
    pub(crate) scale: Option<u8>,
    // pub xml_info: bool, // todo!()
    // pub udt_info: bool, // todo!()
}

impl TypeInfo {

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = [0u8; 10];
        let n = self.encode(&mut bytes);
        bytes[..n].to_vec()
    }

    pub fn to_tsql(&self) -> String {
        match self.dtype {
            DataType::ZeroLength(_) => "null".into(),
            DataType::Fixed(t) => match t {
                FixedLengthDataType::Int1 => "tinyint".into(),
                FixedLengthDataType::Int2 => "smallint".into(),
                FixedLengthDataType::Int4 => "int".into(),
                FixedLengthDataType::Int8 => "bigint".into(),
                FixedLengthDataType::Bit => "bit".into(),
                FixedLengthDataType::Flt4 => "real".into(),
                FixedLengthDataType::Flt8 => "float".into(),
                FixedLengthDataType::Money4 => "smallmoney".into(),
                FixedLengthDataType::Money => "money".into(),
                FixedLengthDataType::DateTim4 => "smalldatetime".into(),
                FixedLengthDataType::DateTime => "datetime".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                FixedLengthDataType::Decimal => "decimal".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                FixedLengthDataType::Numeric => "numeric".into(),
            },
            DataType::Variable(t) => match t {
                VariableLengthDataType::Guid => "uniqueidentifier".into(),
                VariableLengthDataType::IntN => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Byte(1)) => "tinyint".into(),
                    Some(TypeInfoVarLen::Byte(2)) => "smallint".into(),
                    Some(TypeInfoVarLen::Byte(4)) => "int".into(),
                    Some(TypeInfoVarLen::Byte(8)) => "bigint".into(),
                    _ => "int".into(),
                },
                VariableLengthDataType::BitN => "bit".into(),
                VariableLengthDataType::FltN => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Byte(4)) => "real".into(),
                    _ => "float".into(),
                },
                VariableLengthDataType::MoneyN => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Byte(4)) => "smallmoney".into(),
                    _ => "money".into(),
                },
                VariableLengthDataType::DateTimN => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Byte(4)) => "smalldatetime".into(),
                    _ => "datetime".into(),
                },
                VariableLengthDataType::DecimalN => match (self.precision, self.scale) {
                    (Some(p), Some(s)) => format!("decimal({},{})", p, s),
                    _ => "decimal".into(),
                },
                VariableLengthDataType::NumericN => match (self.precision, self.scale) {
                    (Some(p), Some(s)) => format!("numeric({},{})", p, s),
                    _ => "numeric".into(),
                },
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::DateN => "date".into(),
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::TimeN => match self.scale {
                    Some(s) => format!("time({})", s),
                    None => "time".into(),
                },
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::DateTime2N => match self.scale {
                    Some(s) => format!("datetime2({})", s),
                    None => "datetime2".into(),
                },
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::DateTimeOffsetN => match self.scale {
                    Some(s) => format!("datetimeoffset({})", s),
                    None => "datetimeoffset".into(),
                },
                VariableLengthDataType::NVarChar => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(0xffff)) => "nvarchar(max)".into(),
                    Some(TypeInfoVarLen::Ushort(n)) => format!("nvarchar({})", n / 2),
                    _ => "nvarchar(4000)".into(),
                },
                VariableLengthDataType::NChar => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(n)) => format!("nchar({})", n / 2),
                    _ => "nchar(1)".into(),
                },
                VariableLengthDataType::BigVarChar => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(0xffff)) => "varchar(max)".into(),
                    Some(TypeInfoVarLen::Ushort(n)) => format!("varchar({})", n),
                    _ => "varchar(8000)".into(),
                },
                VariableLengthDataType::BigChar => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(n)) => format!("char({})", n),
                    _ => "char(1)".into(),
                },
                VariableLengthDataType::BigVarBinary => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(0xffff)) => "varbinary(max)".into(),
                    Some(TypeInfoVarLen::Ushort(n)) => format!("varbinary({})", n),
                    _ => "varbinary(8000)".into(),
                },
                VariableLengthDataType::BigBinary => match self.dtype_max_len {
                    Some(TypeInfoVarLen::Ushort(n)) => format!("binary({})", n),
                    _ => "binary(1)".into(),
                },
                VariableLengthDataType::Text => "text".into(),
                VariableLengthDataType::NText => "ntext".into(),
                VariableLengthDataType::Image => "image".into(),
                #[cfg(feature = "tds7.2")]
                VariableLengthDataType::SsVariant => "sql_variant".into(),
                #[cfg(feature = "tds7.2")]
                VariableLengthDataType::Xml => "xml".into(),
                #[cfg(feature = "tds7.2")]
                VariableLengthDataType::Udt => "udt".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                VariableLengthDataType::Char => "char".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                VariableLengthDataType::VarChar => "varchar".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                VariableLengthDataType::Binary => "binary".into(),
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                VariableLengthDataType::VarBinary => "varbinary".into(),
                VariableLengthDataType::Json => "json".into(),
                VariableLengthDataType::Vector => "vector".into(),
            },
        }
    }
}

impl TypeInfoBuilder {
    fn validate(&self) -> Result<(), String> {
        if matches!(
            self.dtype_max_len,
            Some(Some(TypeInfoVarLen::Ushort(0xffff)))
        ) {
            match self.dtype {
                Some(DataType::Variable(
                    VariableLengthDataType::NVarChar
                    | VariableLengthDataType::BigVarChar
                    | VariableLengthDataType::BigVarBinary,
                )) => {}
                _ => return Err("".into()),
            }
        }

        //todo: max length 4000 and 8000
        Ok(())
    }
}

// TYPE_VARLEN  = BYTELEN / USHORTCHARBINLE / LONGLEN
#[derive(Debug, Clone, Copy)]
pub enum TypeInfoVarLen {
    Byte(u8),
    Ushort(u16),
    Long(u32),
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Collation {
    pub(crate) lcid: u32,
    pub(crate) f_ignore_case: bool,
    pub(crate) f_ignore_accent: bool,
    pub(crate) f_ignore_width: bool,
    pub(crate) f_ignore_kana: bool,
    pub(crate) f_binary: bool,
    pub(crate) f_binary2: bool,
    pub(crate) f_utf8: bool,
    pub(crate) version: u8,
    pub(crate) sort_id: u8,
}

impl core::fmt::Display for Collation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, ".... .... .... {:04b} {:04b} {:04b} {:04b} {:04b} = LCID: 0x{:05x}",
            (self.lcid >> 16) & 0xf,
            (self.lcid >> 12) & 0xf,
            (self.lcid >> 8) & 0xf,
            (self.lcid >> 4) & 0xf,
            self.lcid & 0xf,
            self.lcid
        )?;
        writeln!(f, ".... .... ...{} .... .... .... .... .... = Ignore case: {}", self.f_ignore_case as u8, self.f_ignore_case)?;
        writeln!(f, ".... .... ..{}. .... .... .... .... .... = Ignore accent: {}", self.f_ignore_accent as u8, self.f_ignore_accent)?;
        writeln!(f, ".... .... .{}.. .... .... .... .... .... = Ignore kana: {}", self.f_ignore_kana as u8, self.f_ignore_kana)?;
        writeln!(f, ".... .... {}... .... .... .... .... .... = Ignore width: {}", self.f_ignore_width as u8, self.f_ignore_width)?;
        writeln!(f, ".... ...{} .... .... .... .... .... .... = Binary: {}", self.f_binary as u8, self.f_binary)?;
        writeln!(f, ".... ..{}. .... .... .... .... .... .... = Binary2: {}", self.f_binary2 as u8, self.f_binary2)?;
        writeln!(f, ".... .{}.. .... .... .... .... .... .... = UTF-8: {}", self.f_utf8 as u8, self.f_utf8)?;
        writeln!(f, "{:04b} .... .... .... .... .... .... .... = Version: {}", self.version & 0xf, self.version)?;
        write!(f, "SortId: {}", match Self::SORT_ID_NAMES_LTU[self.sort_id as usize] {
            Some(name) => format!("{} ({})", name, self.sort_id),
            None => format!("{}", self.sort_id),
        })
    }
}

#[allow(non_upper_case_globals)]
impl Collation {
    fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 { return None; }
        let sort_id = bytes[4];
        let raw = r_u32_le(bytes, 0);
        let lcid = raw & 0xfffff;
        let flags = (raw >> 20) as u8;
        let version = ((raw >> 28) & 0xf) as u8;
        Some(Self {
            lcid,
            f_ignore_case: flags & 0x01 != 0,
            f_ignore_accent: flags >> 1 & 0x01 != 0,
            f_ignore_width: flags >> 2 & 0x01 != 0,
            f_ignore_kana: flags >> 3 & 0x01 != 0,
            f_binary: flags >> 4 & 0x01 != 0,
            f_binary2: flags >> 5 & 0x01 != 0,
            f_utf8: flags >> 6 & 0x01 != 0,
            version,
            sort_id,
        })
    }

    pub const SQL_Latin1_General_Cp437_CS_AS_KI_WI: u8 = 31;
    pub const SQL_Latin1_General_Cp437_CI_AS_KI_WI: u8 = 32;
    pub const SQL_Latin1_General_Pref_Cp437_CI_AS_KI_WI: u8 = 33;
    pub const SQL_Latin1_General_Cp437_CI_AI_KI_WI: u8 = 34;
    pub const SQL_Latin1_General_Cp437_BIN: u8 = 40;
    pub const SQL_Latin1_General_Cp850_BIN: u8 = 40;
    pub const SQL_Latin1_General_Cp850_CS_AS_KI_WI: u8 = 41;
    pub const SQL_Latin1_General_Cp850_CI_AS_KI_WI: u8 = 42;
    pub const SQL_Latin1_General_Cp850_CI_AI_KI_WI: u8 = 44;
    pub const SQL_Latin1_General_Pref_Cp850_CI_AS_KI_WI: u8 = 44;
    pub const SQL_1xCompat_Cp850_CI_AS_KI_WI: u8 = 49;
    pub const SQL_Latin1_General_Cp1_CS_AS_KI_WI: u8 = 51;
    pub const SQL_Latin1_General_Cp1_CI_AS_KI_WI: u8 = 52;
    pub const SQL_Latin1_General_Pref_Cp1_CI_AS_KI_WI: u8 = 53;
    pub const SQL_Latin1_General_Cp1_CI_AI_KI_WI: u8 = 54;
    pub const SQL_AltDiction_Cp850_CS_AS_KI_WI: u8 = 55;
    pub const SQL_AltDiction_Pref_Cp850_CI_AS_KI_WI: u8 = 56;
    pub const SQL_AltDiction_Cp850_CI_AI_KI_WI: u8 = 57;
    pub const SQL_Scandainavian_Pref_Cp850_CI_AS_KI_WI: u8 = 58;
    pub const SQL_Scandainavian_Cp850_CS_AS_KI_WI: u8 = 59;
    pub const SQL_Scandainavian_Cp850_CI_AS_KI_WI: u8 = 60;
    pub const SQL_AltDiction_Cp850_CI_AS_KI_WI: u8 = 61;
    pub const SQL_Latin1_General_1250_BIN: u8 = 80;
    pub const SQL_Latin1_General_Cp1250_CS_AS_KI_WI: u8 = 81;
    pub const SQL_Latin1_General_Cp1250_CI_AS_KI_WI: u8 = 82;
    pub const SQL_Czech_Cp1250_CS_AS_KI_WI: u8 = 83;
    pub const SQL_Czech_Cp1250_CI_AS_KI_WI: u8 = 84;
    pub const SQL_Hungarian_Cp1250_CS_AS_KI_WI: u8 = 85;
    pub const SQL_Hungarian_Cp1250_CI_AS_KI_WI: u8 = 86;
    pub const SQL_Polish_Cp1250_CS_AS_KI_WI: u8 = 87;
    pub const SQL_Polish_Cp1250_CI_AS_KI_WI: u8 = 88;
    pub const SQL_Romanian_Cp1250_CS_AS_KI_WI: u8 = 89;
    pub const SQL_Romanian_Cp1250_CI_AS_KI_WI: u8 = 90;
    pub const SQL_Croatian_Cp1250_CS_AS_KI_WI: u8 = 91;
    pub const SQL_Croatian_Cp1250_CI_AS_KI_WI: u8 = 92;
    pub const SQL_Slovak_Cp1250_CS_AS_KI_WI: u8 = 93;
    pub const SQL_Slovak_Cp1250_CI_AS_KI_WI: u8 = 94;
    pub const SQL_Slovenian_Cp1250_CS_AS_KI_WI: u8 = 95;
    pub const SQL_Slovenian_Cp1250_CI_AS_KI_WI: u8 = 96;
    pub const SQL_Latin1_General_1251_BIN: u8 = 104;
    pub const SQL_Latin1_General_Cp1251_CS_AS_KI_WI: u8 = 105;
    pub const SQL_Latin1_General_Cp1251_CI_AS_KI_WI: u8 = 106;
    pub const SQL_Ukrainian_Cp1251_CS_AS_KI_WI: u8 = 107;
    pub const SQL_Ukrainian_Cp1251_CI_AS_KI_WI: u8 = 108;
    pub const SQL_Latin1_General_1253_BIN: u8 = 112;
    pub const SQL_Latin1_General_Cp1253_CS_AS_KI_WI: u8 = 113;
    pub const SQL_Latin1_General_Cp1253_CI_AS_KI_WI: u8 = 114;
    /// duplicate of [`Self::SQL_Latin1_General_Cp1253_CI_AS_KI_WI`]
    /// <https://learn.microsoft.com/en-us/previous-versions/sql/sql-server-2008-r2/ms144250(v=sql.105)>
    pub const SQL_Latin1_General_Cp1253_CI_AS_KI_WI_A: u8 = 121;
    pub const SQL_Latin1_General_Cp1253_CI_AI_KI_WI: u8 = 124;
    pub const SQL_Latin1_General_1254_BIN: u8 = 128;
    pub const SQL_Latin1_General_Cp1254_CS_AS_KI_WI: u8 = 129;
    pub const SQL_Latin1_General_Cp1254_CI_AS_KI_WI: u8 = 130;
    pub const SQL_Latin1_General_1255_BIN: u8 = 136;
    pub const SQL_Latin1_General_Cp1255_CS_AS_KI_WI: u8 = 137;
    pub const SQL_Latin1_General_Cp1255_CI_AS_KI_WI: u8 = 138;
    pub const SQL_Latin1_General_1256_BIN: u8 = 144;
    pub const SQL_Latin1_General_Cp1256_CS_AS_KI_WI: u8 = 145;
    pub const SQL_Latin1_General_Cp1256_CI_AS_KI_WI: u8 = 146;
    pub const SQL_Latin1_General_1257_BIN: u8 = 152;
    pub const SQL_Latin1_General_Cp1257_CS_AS_KI_WI: u8 = 153;
    pub const SQL_Latin1_General_Cp1257_CI_AS_KI_WI: u8 = 154;
    pub const SQL_Estonian_Cp1257_CS_AS_KI_WI: u8 = 155;
    pub const SQL_Estonian_Cp1257_CI_AS_KI_WI: u8 = 156;
    pub const SQL_Latvian_Cp1257_CS_AS_KI_WI: u8 = 157;
    pub const SQL_Latvian_Cp1257_CI_AS_KI_WI: u8 = 158;
    pub const SQL_Lithuanian_Cp1257_CS_AS_KI_WI: u8 = 159;
    pub const SQL_Lithuanian_Cp1257_CI_AS_KI_WI: u8 = 160;
    pub const SQL_Danish_Pref_Cp1_CI_AS_KI_WI: u8 = 183;
    pub const SQL_SwedishPhone_Pref_Cp1_CI_AS_KI_WI: u8 = 184;
    pub const SQL_SwedishStd_Pref_Cp1_CI_AS_KI_WI: u8 = 185;
    pub const SQL_Icelandic_Pref_Cp1_CI_AS_KI_WI: u8 = 186;

    pub const SORT_ID_NAMES_LTU: [Option<&'static str>; 256] = {
        let mut t = [None; 256];
        t[31] = Some("SQL_Latin1_General_Cp437_CS_AS_KI_WI");
        t[32] = Some("SQL_Latin1_General_Cp437_CI_AS_KI_WI");
        t[33] = Some("SQL_Latin1_General_Pref_Cp437_CI_AS_KI_WI");
        t[34] = Some("SQL_Latin1_General_Cp437_CI_AI_KI_WI");
        t[40] = Some("SQL_Latin1_General_Cp437_BIN / SQL_Latin1_General_Cp850_BIN");
        t[41] = Some("SQL_Latin1_General_Cp850_CS_AS_KI_WI");
        t[42] = Some("SQL_Latin1_General_Cp850_CI_AS_KI_WI");
        t[44] = Some(
            "SQL_Latin1_General_Cp850_CI_AI_KI_WI / SQL_Latin1_General_Pref_Cp850_CI_AS_KI_WI",
        );
        t[49] = Some("SQL_1xCompat_Cp850_CI_AS_KI_WI");
        t[51] = Some("SQL_Latin1_General_Cp1_CS_AS_KI_WI");
        t[52] = Some("SQL_Latin1_General_Cp1_CI_AS_KI_WI");
        t[53] = Some("SQL_Latin1_General_Pref_Cp1_CI_AS_KI_WI");
        t[54] = Some("SQL_Latin1_General_Cp1_CI_AI_KI_WI");
        t[55] = Some("SQL_AltDiction_Cp850_CS_AS_KI_WI");
        t[56] = Some("SQL_AltDiction_Pref_Cp850_CI_AS_KI_WI");
        t[57] = Some("SQL_AltDiction_Cp850_CI_AI_KI_WI");
        t[58] = Some("SQL_Scandainavian_Pref_Cp850_CI_AS_KI_WI");
        t[59] = Some("SQL_Scandainavian_Cp850_CS_AS_KI_WI");
        t[60] = Some("SQL_Scandainavian_Cp850_CI_AS_KI_WI");
        t[61] = Some("SQL_AltDiction_Cp850_CI_AS_KI_WI");
        t[80] = Some("SQL_Latin1_General_1250_BIN");
        t[81] = Some("SQL_Latin1_General_Cp1250_CS_AS_KI_WI");
        t[82] = Some("SQL_Latin1_General_Cp1250_CI_AS_KI_WI");
        t[83] = Some("SQL_Czech_Cp1250_CS_AS_KI_WI");
        t[84] = Some("SQL_Czech_Cp1250_CI_AS_KI_WI");
        t[85] = Some("SQL_Hungarian_Cp1250_CS_AS_KI_WI");
        t[86] = Some("SQL_Hungarian_Cp1250_CI_AS_KI_WI");
        t[87] = Some("SQL_Polish_Cp1250_CS_AS_KI_WI");
        t[88] = Some("SQL_Polish_Cp1250_CI_AS_KI_WI");
        t[89] = Some("SQL_Romanian_Cp1250_CS_AS_KI_WI");
        t[90] = Some("SQL_Romanian_Cp1250_CI_AS_KI_WI");
        t[91] = Some("SQL_Croatian_Cp1250_CS_AS_KI_WI");
        t[92] = Some("SQL_Croatian_Cp1250_CI_AS_KI_WI");
        t[93] = Some("SQL_Slovak_Cp1250_CS_AS_KI_WI");
        t[94] = Some("SQL_Slovak_Cp1250_CI_AS_KI_WI");
        t[95] = Some("SQL_Slovenian_Cp1250_CS_AS_KI_WI");
        t[96] = Some("SQL_Slovenian_Cp1250_CI_AS_KI_WI");
        t[104] = Some("SQL_Latin1_General_1251_BIN");
        t[105] = Some("SQL_Latin1_General_Cp1251_CS_AS_KI_WI");
        t[106] = Some("SQL_Latin1_General_Cp1251_CI_AS_KI_WI");
        t[107] = Some("SQL_Ukrainian_Cp1251_CS_AS_KI_WI");
        t[108] = Some("SQL_Ukrainian_Cp1251_CI_AS_KI_WI");
        t[112] = Some("SQL_Latin1_General_1253_BIN");
        t[113] = Some("SQL_Latin1_General_Cp1253_CS_AS_KI_WI");
        t[114] = Some("SQL_Latin1_General_Cp1253_CI_AS_KI_WI");
        t[121] = Some("SQL_Latin1_General_Cp1253_CI_AS_KI_WI");
        t[124] = Some("SQL_Latin1_General_Cp1253_CI_AI_KI_WI");
        t[128] = Some("SQL_Latin1_General_1254_BIN");
        t[129] = Some("SQL_Latin1_General_Cp1254_CS_AS_KI_WI");
        t[130] = Some("SQL_Latin1_General_Cp1254_CI_AS_KI_WI");
        t[136] = Some("SQL_Latin1_General_1255_BIN");
        t[137] = Some("SQL_Latin1_General_Cp1255_CS_AS_KI_WI");
        t[138] = Some("SQL_Latin1_General_Cp1255_CI_AS_KI_WI");
        t[144] = Some("SQL_Latin1_General_1256_BIN");
        t[145] = Some("SQL_Latin1_General_Cp1256_CS_AS_KI_WI");
        t[146] = Some("SQL_Latin1_General_Cp1256_CI_AS_KI_WI");
        t[152] = Some("SQL_Latin1_General_1257_BIN");
        t[153] = Some("SQL_Latin1_General_Cp1257_CS_AS_KI_WI");
        t[154] = Some("SQL_Latin1_General_Cp1257_CI_AS_KI_WI");
        t[155] = Some("SQL_Estonian_Cp1257_CS_AS_KI_WI");
        t[156] = Some("SQL_Estonian_Cp1257_CI_AS_KI_WI");
        t[157] = Some("SQL_Latvian_Cp1257_CS_AS_KI_WI");
        t[158] = Some("SQL_Latvian_Cp1257_CI_AS_KI_WI");
        t[159] = Some("SQL_Lithuanian_Cp1257_CS_AS_KI_WI");
        t[160] = Some("SQL_Lithuanian_Cp1257_CI_AS_KI_WI");
        t[183] = Some("SQL_Danish_Pref_Cp1_CI_AS_KI_WI");
        t[184] = Some("SQL_SwedishPhone_Pref_Cp1_CI_AS_KI_WI");
        t[185] = Some("SQL_SwedishStd_Pref_Cp1_CI_AS_KI_WI");
        t[186] = Some("SQL_Icelandic_Pref_Cp1_CI_AS_KI_WI");
        t
    };
}

impl<'a> From<TypeInfoSpan<'a>> for TypeInfo {
    fn from(span: TypeInfoSpan<'a>) -> Self {
        let d = &span.bytes[1..];
        match span.dtype {
            DataType::ZeroLength(_) | DataType::Fixed(_) => Self {
                dtype: span.dtype,
                dtype_max_len: None,
                collation: None,
                precision: None,
                scale: None,
            },
            DataType::Variable(t) => match t {
                VariableLengthDataType::Guid
                | VariableLengthDataType::IntN
                | VariableLengthDataType::BitN
                | VariableLengthDataType::FltN
                | VariableLengthDataType::MoneyN
                | VariableLengthDataType::DateTimN => Self {
                    dtype: span.dtype,
                    dtype_max_len: Some(TypeInfoVarLen::Byte(d[0])),
                    collation: None,
                    precision: None,
                    scale: None,
                },
                VariableLengthDataType::DecimalN | VariableLengthDataType::NumericN => Self {
                    dtype: span.dtype,
                    scale: Some(d[2]),
                    dtype_max_len: Some(TypeInfoVarLen::Byte(d[0])),
                    collation: None,
                    precision: Some(d[1]),
                },
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::DateN => Self {
                    dtype: span.dtype,
                    dtype_max_len: None,
                    collation: None,
                    precision: None,
                    scale: None,
                },
                #[cfg(feature = "tds7.3")]
                VariableLengthDataType::TimeN
                | VariableLengthDataType::DateTime2N
                | VariableLengthDataType::DateTimeOffsetN => Self {
                    dtype: span.dtype,
                    dtype_max_len: None,
                    collation: None,
                    precision: None,
                    scale: Some(d[0]),
                },
                VariableLengthDataType::BigVarBinary | VariableLengthDataType::BigBinary => Self {
                    dtype: span.dtype,
                    dtype_max_len: Some(TypeInfoVarLen::Ushort(r_u16_le(d, 0))),
                    collation: None,
                    precision: None,
                    scale: None,
                },
                VariableLengthDataType::BigVarChar
                | VariableLengthDataType::BigChar
                | VariableLengthDataType::NVarChar
                | VariableLengthDataType::NChar => Self {
                    dtype: span.dtype,
                    collation: Collation::new(&d[2..7]),
                    dtype_max_len: Some(TypeInfoVarLen::Ushort(r_u16_le(d, 0))),
                    precision: None,
                    scale: None,
                },
                VariableLengthDataType::Text | VariableLengthDataType::Image => Self {
                    dtype: span.dtype,
                    dtype_max_len: Some(TypeInfoVarLen::Long(r_u32_le(d, 0))),
                    collation: None,
                    precision: None,
                    scale: None,
                },
                VariableLengthDataType::NText => Self {
                    dtype: span.dtype,
                    collation: Collation::new(&d[4..9]),
                    dtype_max_len: Some(TypeInfoVarLen::Long(r_u32_le(d, 0))),
                    precision: None,
                    scale: None,
                },
                #[cfg(feature = "legacy")]
                #[allow(deprecated)]
                VariableLengthDataType::Char
                | VariableLengthDataType::VarChar
                | VariableLengthDataType::Binary
                | VariableLengthDataType::VarBinary => unimplemented!(),
                #[cfg(feature = "tds7.2")]
                VariableLengthDataType::Xml | VariableLengthDataType::Udt => unimplemented!(),
                #[cfg(feature = "tds7.2")]
                VariableLengthDataType::SsVariant => unimplemented!(),
                VariableLengthDataType::Json => unimplemented!(),
                VariableLengthDataType::Vector => Self {
                    dtype: span.dtype,
                    dtype_max_len: Some(TypeInfoVarLen::Ushort(r_u16_le(d, 0))),
                    collation: None,
                    precision: None,
                    scale: Some(d[2]),
                },
            },
        }
    }
}

pub fn to_dtype_bytes<'a>(bytes: &mut &'a [u8], type_byte: u8) -> Option<&'a [u8]> {
    let n = walk(bytes, 0, DTYPE_LUT[type_byte as usize].stride)?;
    if n > bytes.len() {
        return None;
    }
    let col = &bytes[..n];
    *bytes = &bytes[n..];
    Some(col)
}

/// MS-TDS §2.2.5.4
/// DO NOT MODIFY
const _: () = {
    // Zero-Length
    const NULLTYPE: u8 = 0x1f; // Null
    assert!(ZeroLengthDataType::Null as u8 == NULLTYPE);
    assert!(ZeroLengthDataType::COUNT == 1);
    // Fixed-Length
    const INT1TYPE: u8 = 0x30; // TinyInt
    const BITTYPE: u8 = 0x32; // Bit
    const INT2TYPE: u8 = 0x34; // SmallInt
    const INT4TYPE: u8 = 0x38; // Int
    const DATETIM4TYPE: u8 = 0x3A; // SmallDateTime
    const FLT4TYPE: u8 = 0x3B; // Real
    const MONEYTYPE: u8 = 0x3C; // Money
    const DATETIMETYPE: u8 = 0x3D; // DateTime
    const FLT8TYPE: u8 = 0x3E; // Float
    const MONEY4TYPE: u8 = 0x7A; // SmallMoney
    const INT8TYPE: u8 = 0x7F; // BigInt
    #[cfg(feature = "legacy")]
    #[deprecated]
    const DECIMALTYPE: u8 = 0x37; // Decimal (legacy support)
    #[cfg(feature = "legacy")]
    #[deprecated]
    const NUMERICTYPE: u8 = 0x3F; // Numeric (legacy support)
    assert!(FixedLengthDataType::Int1 as u8 == INT1TYPE);
    assert!(FixedLengthDataType::Bit as u8 == BITTYPE);
    assert!(FixedLengthDataType::Int2 as u8 == INT2TYPE);
    assert!(FixedLengthDataType::Int4 as u8 == INT4TYPE);
    assert!(FixedLengthDataType::DateTim4 as u8 == DATETIM4TYPE);
    assert!(FixedLengthDataType::Flt4 as u8 == FLT4TYPE);
    assert!(FixedLengthDataType::Money as u8 == MONEYTYPE);
    assert!(FixedLengthDataType::DateTime as u8 == DATETIMETYPE);
    assert!(FixedLengthDataType::Flt8 as u8 == FLT8TYPE);
    assert!(FixedLengthDataType::Money4 as u8 == MONEY4TYPE);
    assert!(FixedLengthDataType::Int8 as u8 == INT8TYPE);
    #[cfg(feature = "legacy")]
    assert!(FixedLengthDataType::Decimal as u8 == DECIMALTYPE);
    #[cfg(feature = "legacy")]
    assert!(FixedLengthDataType::Numeric as u8 == NUMERICTYPE);
    #[cfg(not(feature = "legacy"))]
    assert!(FixedLengthDataType::COUNT == 11);
    // Variable-Length
    const GUIDTYPE: u8 = 0x24; // UniqueIdentifier
    const INTNTYPE: u8 = 0x26; // (see below)
    const BITNTYPE: u8 = 0x68; // (see below)
    const DECIMALNTYPE: u8 = 0x6A; // Decimal
    const NUMERICNTYPE: u8 = 0x6C; // Numeric
    const FLTNTYPE: u8 = 0x6D; // (see below)
    const MONEYNTYPE: u8 = 0x6E; // (see below)
    const DATETIMNTYPE: u8 = 0x6F; // (see below)
    #[cfg(feature = "tds7.3")]
    const DATENTYPE: u8 = 0x28; // (introduced in TDS 7.3)
    #[cfg(feature = "tds7.3")]
    const TIMENTYPE: u8 = 0x29; // (introduced in TDS 7.3)
    #[cfg(feature = "tds7.3")]
    const DATETIME2NTYPE: u8 = 0x2A; // (introduced in TDS 7.3)
    #[cfg(feature = "tds7.3")]
    const DATETIMEOFFSETNTYPE: u8 = 0x2B; // (introduced in TDS 7.3)
    #[cfg(feature = "legacy")]
    const CHARTYPE: u8 = 0x2F; // Char (legacy support)
    #[cfg(feature = "legacy")]
    const VARCHARTYPE: u8 = 0x27; // VarChar (legacy support)
    #[cfg(feature = "legacy")]
    const BINARYTYPE: u8 = 0x2D; // Binary (legacy support)
    #[cfg(feature = "legacy")]
    const VARBINARYTYPE: u8 = 0x25; // VarBinary (legacy support)
    const BIGVARBINARYTYPE: u8 = 0xA5; // VarBinary
    const BIGVARCHARTYPE: u8 = 0xA7; // VarChar
    const BIGBINARYTYPE: u8 = 0xAD; // Binary
    const BIGCHARTYPE: u8 = 0xAF; // Char
    const NVARCHARTYPE: u8 = 0xE7; // NVarChar
    const NCHARTYPE: u8 = 0xEF; // NChar
    #[cfg(feature = "tds7.2")]
    const XMLTYPE: u8 = 0xF1; // XML (introduced in TDS 7.2)
    #[cfg(feature = "tds7.2")]
    const UDTTYPE: u8 = 0xF0; // CLR UDT (introduced in TDS 7.2)
    const TEXTTYPE: u8 = 0x23; // Text
    const IMAGETYPE: u8 = 0x22; // Image
    const NTEXTTYPE: u8 = 0x63; // NText
    #[cfg(feature = "tds7.2")]
    const SSVARIANTTYPE: u8 = 0x62; // sql_variant (introduced in TDS 7.2)
    const JSONTYPE: u8 = 0xF4;
    const VECTORTYPE: u8 = 0xF5;

    assert!(VariableLengthDataType::Guid as u8 == GUIDTYPE);
    assert!(VariableLengthDataType::IntN as u8 == INTNTYPE);
    assert!(VariableLengthDataType::BitN as u8 == BITNTYPE);
    assert!(VariableLengthDataType::DecimalN as u8 == DECIMALNTYPE);
    assert!(VariableLengthDataType::NumericN as u8 == NUMERICNTYPE);
    assert!(VariableLengthDataType::FltN as u8 == FLTNTYPE);
    assert!(VariableLengthDataType::MoneyN as u8 == MONEYNTYPE);
    assert!(VariableLengthDataType::DateTimN as u8 == DATETIMNTYPE);
    #[cfg(feature = "tds7.3")]
    assert!(VariableLengthDataType::DateN as u8 == DATENTYPE);
    #[cfg(feature = "tds7.3")]
    assert!(VariableLengthDataType::TimeN as u8 == TIMENTYPE);
    #[cfg(feature = "tds7.3")]
    assert!(VariableLengthDataType::DateTime2N as u8 == DATETIME2NTYPE);
    #[cfg(feature = "tds7.3")]
    assert!(VariableLengthDataType::DateTimeOffsetN as u8 == DATETIMEOFFSETNTYPE);
    #[cfg(feature = "legacy")]
    assert!(VariableLengthDataType::Char as u8 == CHARTYPE);
    #[cfg(feature = "legacy")]
    assert!(VariableLengthDataType::VarChar as u8 == VARCHARTYPE);
    #[cfg(feature = "legacy")]
    assert!(VariableLengthDataType::Binary as u8 == BINARYTYPE);
    #[cfg(feature = "legacy")]
    assert!(VariableLengthDataType::VarBinary as u8 == VARBINARYTYPE);
    assert!(VariableLengthDataType::BigVarBinary as u8 == BIGVARBINARYTYPE);
    assert!(VariableLengthDataType::BigVarChar as u8 == BIGVARCHARTYPE);
    assert!(VariableLengthDataType::BigBinary as u8 == BIGBINARYTYPE);
    assert!(VariableLengthDataType::BigChar as u8 == BIGCHARTYPE);
    assert!(VariableLengthDataType::NVarChar as u8 == NVARCHARTYPE);
    assert!(VariableLengthDataType::NChar as u8 == NCHARTYPE);
    #[cfg(feature = "tds7.2")]
    assert!(VariableLengthDataType::Xml as u8 == XMLTYPE);
    #[cfg(feature = "tds7.2")]
    assert!(VariableLengthDataType::Udt as u8 == UDTTYPE);
    assert!(VariableLengthDataType::Text as u8 == TEXTTYPE);
    assert!(VariableLengthDataType::Image as u8 == IMAGETYPE);
    assert!(VariableLengthDataType::NText as u8 == NTEXTTYPE);
    #[cfg(feature = "tds7.2")]
    assert!(VariableLengthDataType::SsVariant as u8 == SSVARIANTTYPE);
    assert!(VariableLengthDataType::Json as u8 == JSONTYPE);
    assert!(VariableLengthDataType::Vector as u8 == VECTORTYPE);
    #[cfg(feature = "tds7.3")]
    assert!(VariableLengthDataType::COUNT == 26);
    #[cfg(all(feature = "tds7.2", not(feature = "tds7.3")))]
    assert!(VariableLengthDataType::COUNT == 22);
    #[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
    assert!(VariableLengthDataType::COUNT == 19);
};
