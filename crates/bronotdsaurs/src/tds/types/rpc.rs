#![allow(unused)]
use crate::tds::prelude::*;

tds_packet_header!(RPCReqBatchHeader, ClientMessageType::RemoteProcedureCall);

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct RPCReqBatch {
    pub(crate) all_headers: AllHeaders,
    pub(crate) name_len_proc_id: NameLenProcId,
    pub(crate) option_flags: OptionFlags,
    #[cfg(feature = "tds7.4")]
    pub(crate) enclave_package: Vec<u8>,
    pub(crate) parameter_data: Vec<ParameterData>, // repeated once per parameter
}

impl RPCReqBatch {
    #[cfg(not(feature = "tds7.2"))]
    pub const BATCH_FLAG: u8 = 0x80;
    #[cfg(feature = "tds7.2")]
    pub const BATCH_FLAG: u8 = 0xff;
    #[cfg(feature = "tds7.2")]
    pub const NO_EXEC_FLAG: u8 = 0xfe;
}

#[derive(Debug, Clone)]
pub struct RPCRequest {
    pub(crate) all_headers: AllHeaders,
    rpc_req_batch: RPCReqBatch,
}

impl RPCRequest {
    pub const MAX_PROC_NAME_LENGTH: usize = 1046;
}

#[derive(Debug, Clone)]
pub enum NameLenProcId {
    ProcName(ProcName),
    ProcID(ProcId),
}

#[derive(Debug, Clone)]
pub struct ProcName(pub String);

#[derive(Debug)]
pub struct ProcNameTooLong;

impl core::fmt::Display for ProcNameTooLong {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "proc name exceeds {} characters",
            RPCRequest::MAX_PROC_NAME_LENGTH
        )
    }
}

impl core::error::Error for ProcNameTooLong {}

impl ProcName {
    pub fn new(name: String) -> Result<Self, ProcNameTooLong> {
        if name.encode_utf16().count() > RPCRequest::MAX_PROC_NAME_LENGTH {
            return Err(ProcNameTooLong);
        }

        Ok(Self(name))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum ProcId {
    SpCursor = 1,
    SpCursorOpen = 2,
    SpCursorPrepare = 3,
    SpCursorExecute = 4,
    SpCursorPrepExec = 5,
    SpCursorUnprepare = 6,
    SpCursorFetch = 7,
    SpCursorOption = 8,
    SpCursorClose = 9,
    SpExecuteSql = 10,
    SpPrepare = 11,
    SpExecute = 12,
    SpPrepExec = 13,
    SpPrepExecRpc = 14,
    SpUnprepare = 15,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OptionFlags(pub u16);

impl OptionFlags {
    /// OptionFlags constructor
    ///
    /// ### Parameters
    /// - f_with_recomp (bool): `RPC_SENT_WITH_RECOMP` (1)
    /// - f_no_metadata (bool): `NO_META_DATA` (1)
    /// - f_reuse_metadata (bool): `REUSE_META_DATA` (1)
    #[inline]
    pub fn new(f_with_recomp: bool, f_no_metadata: bool, f_reuse_metadata: bool) -> Self {
        let option_flags =
            (f_with_recomp as u16) | (f_no_metadata as u16) << 1 | (f_reuse_metadata as u16) << 2;
        Self(option_flags)
    }

    /// return OptionFlags as a u16
    /// note: call to_le_bytes()
    pub fn as_bytes(&self) -> u16 {
        self.0
    }

    pub const RPC_SENT_WITH_RECOMP: bool = true;
    pub const NO_META_DATA: bool = true;
    /// `1` if the metadata has not changed from the previous call and the server
    /// should reuse its cached metadata. The metadata MUST still be sent.
    pub const REUSE_META_DATA: bool = true;

    /// f_with_recomp option flag bit accessor
    #[inline]
    pub fn f_with_recomp(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// f_no_meta_data option flag bit accessor
    #[inline]
    pub fn f_no_meta_data(&self) -> bool {
        self.0 & 0x02 != 0
    }

    /// f_reuse_meta_data option flag bit accessor
    #[inline]
    pub fn f_reuse_meta_data(&self) -> bool {
        self.0 & 0x04 != 0
    }
}

macro_rules! int_param_dtype {
    ($name: ident, $ty:ty, $dtype:expr) => {
        pub fn $name(name: impl Into<String>, data: $ty) -> Self {
                ParameterData::new(name,
                    StatusFlags::new(false, false, false),
                    TypeInfoBuilder::default()
                    .dtype(DataType::Fixed($dtype))
                    .dtype_max_len(None)
                    .collation(None)
                    .precision(None)
                    .scale(None)
                    .build()
                    .unwrap(),
                data.to_le_bytes().to_vec(),
            )
        }
        
    };
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ParameterData {
    pub(crate) param_meta_data: ParamMetaData,
    pub(crate) param_len_data: Vec<u8>,
    pub(crate) param_cipher_info: Vec<ParamCipherInfo>,
}

impl ParameterData {
    int_param_dtype!(cursor, i32, FixedLengthDataType::Int4);
    int_param_dtype!(int4, i32, FixedLengthDataType::Int4);
    int_param_dtype!(uint4, u32, FixedLengthDataType::Int4);

    pub fn nvarchar(name: impl Into<String>, data: &str) -> Self {
        ParameterData::new(name, 
            StatusFlags::new(false, false, false),
            TypeInfoBuilder::default()
                .dtype(DataType::Variable(VariableLengthDataType::NVarChar))
                .dtype_max_len(Some(TypeInfoVarLen::Ushort(0x1f40)))
                // .dtype_max_len(Some(TypeInfoVarLen::Ushort(0xffff)))
                .collation(Some(Collation::default()))
                .precision(None)
                .scale(None)
                .build()
                .unwrap(),
            data
            .encode_utf16()
            .flat_map(|x| x.to_le_bytes())
            .collect(),
        )
    }

    pub fn option_flags(name: impl Into<String>, data: OptionFlags) -> Self {
        ParameterData::new(name,
        StatusFlags::new(false, false, false),
        TypeInfoBuilder::default()
            .dtype(DataType::Fixed(FixedLengthDataType::Int4))
            .dtype_max_len(None)
            .collation(Some(Collation::default()))
            .precision(None)
            .scale(None)
            .build()
            .unwrap(),
            data.as_bytes().to_le_bytes().to_vec(),
        )
    }

    pub fn new(name: impl Into<String>, status_flags: StatusFlags, type_info: TypeInfo, data: Vec<u8>) -> Self {
        ParameterDataBuilder::default()
            .param_meta_data(
                ParamMetaDataBuilder::default()
                    .name(name.into())
                    .status_flags(status_flags)
                    .type_info(type_info)
                    .build()
                    .unwrap(),
            )
            .param_len_data(data)
            .param_cipher_info(vec![])
            .build()
            .unwrap()
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ParamMetaData {
    pub(crate) name: String,
    pub(crate) status_flags: StatusFlags,
    pub(crate) type_info: TypeInfo,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatusFlags(pub u8);

impl StatusFlags {
    /// StatusFlags constructor
    ///
    /// ### Parameters
    /// - f_by_ref_value (bool): `OUTPUT_BY_REF` (1) or `OUTPUT_BY_VAL` (0) if the parameter is send by reference or value respectively.
    /// - f_default_value (bool): `DEFAULT_PARAMETER_VALUE` (1) if the parameter is passed as default.
    /// - f_encrypted (bool): `ENCRYPTED_PARAMETER` (1) if the parameter is encrypted
    #[inline]
    pub fn new(f_by_ref_value: bool, f_default_value: bool, f_encrypted: bool) -> Self {
        let status_flags =
            (f_by_ref_value as u8) | (f_default_value as u8) << 1 | (f_encrypted as u8) << 3;
        Self(status_flags)
    }

    /// return StatusFlags in bytes
    #[inline]
    pub fn as_bytes(&self) -> u8 {
        self.0
    }

    pub const OUTPUT_BY_REF: bool = true;
    pub const OUTPUT_BY_VAL: bool = false;

    /// f_by_ref status flag bit accessor
    ///
    /// - OUTPUT_BY_REF (or 1) if the parameter is send by reference
    /// - OUTPUT_BY_VAL (or 0) if the parameter is send by value
    #[inline]
    pub fn f_by_ref_value(&self) -> bool {
        self.0 & 0x01 != 0
    }

    pub const DEFAULT_PARAMETER_VALUE: bool = true;

    /// f_default_value status flag bit accessor
    #[inline]
    pub fn f_default_value(&self) -> bool {
        self.0 & 0x02 != 0
    }

    pub const ENCRYPTED_PARAMETER: bool = true;

    /// f_encrypted status flag bit accessor
    #[inline]
    pub fn f_encrypted(&self) -> bool {
        self.0 & 0x08 != 0
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(no_std, setter(strip_option))]
pub struct ParamCipherInfo {
    pub(crate) ty: TypeInfo,
    pub(crate) encryption_algo: u8,       // byte
    pub(crate) algo_name: Option<String>, // b_varchar - if encryption_algo = 0, it must be sent, otherwise, it must not be sent.
    pub(crate) encryption_type: u8,       // byte
    pub(crate) database_id: u32,          // ulong
    pub(crate) cek_id: u32,               // ulong
    pub(crate) cek_version: u32,          // ulong
    pub(crate) cek_md_version: u64,       //ulonglong
    pub(crate) norm_version: u8,          // byte - reserved for future use
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_flag_output_by_ref() {
        let status_flag = StatusFlags::new(StatusFlags::OUTPUT_BY_REF, false, false);
        assert_eq!(status_flag.f_by_ref_value(), StatusFlags::OUTPUT_BY_REF);
        let bytes = status_flag.as_bytes();
        assert_eq!(bytes, 0x1);
    }

    #[test]
    fn test_status_flag_output_by_val() {
        let status_flag = StatusFlags::new(StatusFlags::OUTPUT_BY_VAL, false, false);
        assert_eq!(status_flag.f_by_ref_value(), StatusFlags::OUTPUT_BY_VAL);
        let bytes = status_flag.as_bytes();
        assert_eq!(bytes, 0x0);
    }

    #[test]
    fn test_status_flag_default_parameter_value() {
        let status_flag = StatusFlags::new(false, StatusFlags::DEFAULT_PARAMETER_VALUE, false);
        assert_eq!(
            status_flag.f_default_value(),
            StatusFlags::DEFAULT_PARAMETER_VALUE
        );
        let bytes = status_flag.as_bytes();
        assert_eq!(bytes, 0x2);
    }

    #[test]
    fn test_status_flag_encrypted_parameter_value() {
        let status_flag = StatusFlags::new(false, false, StatusFlags::ENCRYPTED_PARAMETER);
        assert_eq!(status_flag.f_encrypted(), StatusFlags::ENCRYPTED_PARAMETER);
        let bytes = status_flag.as_bytes();
        assert_eq!(bytes, 0x8);
    }

    #[test]
    fn test_option_flag_is_false() {
        let option_flag = OptionFlags::new(false, false, false);
        assert_eq!(
            option_flag.f_with_recomp(),
            !OptionFlags::RPC_SENT_WITH_RECOMP
        );
        assert_eq!(option_flag.f_no_meta_data(), !OptionFlags::NO_META_DATA);
        assert_eq!(
            option_flag.f_reuse_meta_data(),
            !OptionFlags::REUSE_META_DATA
        );
        let bytes = option_flag.as_bytes();
        assert_eq!(bytes, 0x0);
    }

    #[test]
    fn test_option_flag_with_recomp() {
        let option_flag = OptionFlags::new(OptionFlags::RPC_SENT_WITH_RECOMP, false, false);
        assert_eq!(
            option_flag.f_with_recomp(),
            OptionFlags::RPC_SENT_WITH_RECOMP
        );
        let bytes = option_flag.as_bytes();
        assert_eq!(bytes, 0x1);
    }

    #[test]
    fn test_option_flag_no_meta_data() {
        let option_flag = OptionFlags::new(false, OptionFlags::NO_META_DATA, false);
        assert_eq!(option_flag.f_no_meta_data(), OptionFlags::NO_META_DATA);
        let bytes = option_flag.as_bytes();
        assert_eq!(bytes, 0x2);
    }

    #[test]
    fn test_option_flag_reuse_meta_data() {
        let option_flag = OptionFlags::new(false, false, OptionFlags::REUSE_META_DATA);
        assert_eq!(
            option_flag.f_reuse_meta_data(),
            OptionFlags::REUSE_META_DATA
        );
        let bytes = option_flag.as_bytes();
        assert_eq!(bytes, 0x4);
    }
}
