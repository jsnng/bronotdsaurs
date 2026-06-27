use crate::tds::prelude::*;

#[derive(Debug, Clone)]
pub struct ErrorInfoToken {
    pub(crate) ty: DataTokenType,
    pub(crate) length: u16,         // ushort
    pub(crate) number: u32,         // long
    pub(crate) state: u8,           // byte
    pub(crate) class: ErrorClass,   // byte
    pub(crate) msg_text: String,    // us_varchar,
    pub(crate) server_name: String, // b_varchar,
    pub(crate) proc_name: String,
    #[cfg(not(feature = "tds7.3"))]
    pub(crate) line_number: u16, // ushort
    #[cfg(feature = "tds7.3")]
    pub(crate) line_number: u32, // long
}

impl core::fmt::Display for ErrorInfoToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}:{}] {}", self.number(), self.line_number(), self.msg_text())
    }
}

#[derive(Debug, Clone, Copy, DefaultDisplayFormat)]
pub enum ErrorClass {
    Informational,
    DoesNotExistError,
    PossibleDataCorruptionWarning,
    TransactionDeadlockError,
    SecurityError,
    SyntaxError,
    GeneralError,
    UnspecifiedDBEngineProblem,
    NonConfigurableDBLimit,
    OutOfResourcesError,
    SQLStatementEncountedAnError,
    AffectingAllTasks,
    DamagedTableOrIndex,
    DBIntegrityWarning,
    MediaFailure,
}

impl TryFrom<u8> for ErrorClass {
    type Error = DecodeError;
    fn try_from(class: u8) -> Result<Self, Self::Error> {
        match class {
            0..=10 => Ok(ErrorClass::Informational),
            11 => Ok(ErrorClass::DoesNotExistError),
            12 => Ok(ErrorClass::PossibleDataCorruptionWarning),
            13 => Ok(ErrorClass::TransactionDeadlockError),
            14 => Ok(ErrorClass::SecurityError),
            15 => Ok(ErrorClass::SyntaxError),
            16 => Ok(ErrorClass::GeneralError),
            17 => Ok(ErrorClass::OutOfResourcesError),
            18 => Ok(ErrorClass::UnspecifiedDBEngineProblem),
            19 => Ok(ErrorClass::NonConfigurableDBLimit),
            20 => Ok(ErrorClass::SQLStatementEncountedAnError),
            21 => Ok(ErrorClass::AffectingAllTasks),
            22 => Ok(ErrorClass::DamagedTableOrIndex),
            23 => Ok(ErrorClass::DBIntegrityWarning),
            24 => Ok(ErrorClass::MediaFailure),
            _ => Err(DecodeError::InvalidField(format!(
                "ErrorClass: invalid value {}",
                class
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ErrorInfoSpan<'a> {
    pub(crate) bytes: &'a [u8],
    ib_line_number: usize,
    cch_msg_text: usize,
    ib_server_name: usize,
    cch_server_name: usize,
    ib_proc_name: usize,
    cch_proc_name: usize,
}

#[rustfmt::skip]
impl<'a> ErrorInfoSpan<'a> {
    pub const VARIABLE_SPAN_SIZE: usize = 11;

    // Post: if Some(), then bytes.len() is the token size only AND
    // ty() is either Error or Info DataType AND
    // class() is between 11 and 24 for Error or <= 10 for Info.
    #[cfg_attr(kani, kani::ensures(|x: &Result<Self, DecodeError>|
        x.as_ref().map_or(true, |y|
            (y.length() as usize + 3 == bytes.len())
            && ((y.bytes[0] == 0xab && y.bytes[8] <= 10) || (y.bytes[0] == 0xaa && y.bytes[8] >= 11 && y.bytes[8] <= 24))
        )
    ))]
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::VARIABLE_SPAN_SIZE {
            return Err(kani_error_stubbed!(DecodeError::InvalidDataTokenType(format!("ErrorInfoSpan::new() bytes.len()={} < VARIABLE_SPAN_SIZE={}", bytes.len(), Self::VARIABLE_SPAN_SIZE))));
        }

        if r_u16_le(bytes, 1) as usize + 3 != bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("ErrorInfoSpan::new() declared length {} + 3 != bytes.len() {}", r_u16_le(bytes, 1), bytes.len()))));
        }

        if bytes[0] != 0xaa && bytes[0] != 0xab {
            return Err(kani_error_stubbed!(DecodeError::InvalidDataTokenType(format!("ErrorInfoSpan::new() - not 0xaa or 0xab, got {}", bytes[0]))));
        }

        if (bytes[0] == 0xaa && (bytes[8] < 11 || bytes[8] > 24)) || (bytes[0] == 0xab && bytes[8] > 10) {

            return Err(kani_error_stubbed!(DecodeError::InvalidData(format!("ErrorInfoSpan::new() - got {} but class is {}", bytes[0], bytes[8]))));

        }

        let cch_msg_text =  r_u16_le(bytes, 9) as usize * 2;
        let ib_cch_server_name = Self::VARIABLE_SPAN_SIZE + cch_msg_text;
        if ib_cch_server_name >= bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("OutOfBounds: ib_cch_server_name = {}, bytes.len() = {}", ib_cch_server_name, bytes.len()))));
        };
        let cch_server_name = bytes[ib_cch_server_name] as usize * 2;
        let ib_server_name = ib_cch_server_name + 1;
        let ib_cch_proc_name = ib_server_name + cch_server_name;
        if ib_cch_proc_name >= bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("OutOfBounds: ib_cch_proc_name = {}, bytes.len() = {}", ib_cch_proc_name, bytes.len()))));
        };
        let cch_proc_name = bytes[ib_cch_proc_name] as usize * 2;
        let ib_proc_name = ib_cch_proc_name + 1;
        let ib_line_number = ib_proc_name + cch_proc_name;
        #[cfg(not(feature = "tds7.3"))]
        if ib_line_number+size_of::<u16>() > bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("OutOfBounds: ib_line_number = {}, bytes.len() = {}", ib_line_number, bytes.len()))));
        };
        #[cfg(feature = "tds7.3")]
        if ib_line_number+size_of::<u32>() != bytes.len() {
            return Err(kani_error_stubbed!(DecodeError::InvalidLength(format!("OutOfBounds: ib_line_number = {}, bytes.len() = {}", ib_line_number, bytes.len()))));
        };

        Ok(Self { 
            bytes,
            ib_line_number,
            cch_msg_text,
            ib_server_name,
            cch_server_name,
            ib_proc_name,
            cch_proc_name,
        })
    }

    // Post: ty() is either Error or Info.
    #[cfg_attr(kani, kani::ensures(|x: &DataTokenType|
        *x == DataTokenType::Error || *x == DataTokenType::Info
    ))]
    #[inline(always)]
    pub fn ty(&self) -> DataTokenType {
        DataTokenType::from_u8(self.bytes[0]).unwrap()
    }

    #[inline(always)]
    // length gives the token body size only.
    pub fn length(&self) -> u16 {
        r_u16_le(self.bytes, 1)
    }

    #[inline(always)]
    pub fn number(&self) -> u32 {
        r_u32_le(self.bytes, 3)
    }

    #[inline(always)]
    pub fn state(&self) -> u8 { self.bytes[7] }

    #[inline(always)]
    pub fn class(&self) -> u8 { self.bytes[8] }

    #[inline(always)]
    pub fn msg_text(&self) -> NVarCharSpan<'_> {
        NVarCharSpan::new(&self.bytes[Self::VARIABLE_SPAN_SIZE..Self::VARIABLE_SPAN_SIZE + self.cch_msg_text])
    }

    #[inline(always)]
    pub fn server_name(&self) -> NVarCharSpan<'_> {
        NVarCharSpan::new(&self.bytes[self.ib_server_name..self.ib_server_name + self.cch_server_name])
    }

    #[inline(always)]
    pub fn proc_name(&self) -> NVarCharSpan<'_> {
        NVarCharSpan::new(&self.bytes[self.ib_proc_name..self.ib_proc_name + self.cch_proc_name])
    }

    #[cfg(not(feature = "tds7.3"))]
    #[inline(always)]
    pub fn line_number(&self) -> u16 {
        r_u16_le(self.bytes, self.ib_line_number)
    }

    #[cfg(feature = "tds7.3")]
    #[inline(always)]
    pub fn line_number(&self) -> u32 {
        r_u32_le(self.bytes, self.ib_line_number)
    }
}

impl ErrorInfoToken {
    pub fn msg_text(&self) -> &str {
        &self.msg_text
    }
    pub fn number(&self) -> u32 {
        self.number
    }
    #[cfg(not(feature = "tds7.3"))]
    pub fn line_number(&self) -> u16 {
        self.line_number
    }
    #[cfg(feature = "tds7.3")]
    pub fn line_number(&self) -> u32 {
        self.line_number
    }
}

#[cfg(kani)]
#[kani::proof]
fn proof_error_info_span_is_none() {
    let bytes: [u8; 10] = kani::any();
    assert!(ErrorInfoSpan::new(&bytes).is_err())
}

macro_rules! proof_accessor {
    ($name:ident, $method:ident, $check:expr) => {
        #[cfg(kani)]
        #[kani::proof]
        fn $name() {
            let bytes: [u8; 128] = kani::any();
            let slice = kani::slice::any_slice_of_array(&bytes);
            if let Ok(span) = ErrorInfoSpan::new(&slice) {
                assert!(($check)(&span));
            }
        }
    }
}

proof_accessor!(proof_ty, ty, |span: &ErrorInfoSpan<'_>| matches!(span.ty(), DataTokenType::Error | DataTokenType::Info));
proof_accessor!(proof_msg_text, msg_text, |span: &ErrorInfoSpan<'_>| span.msg_text().bytes.len() <= span.bytes.len());
proof_accessor!(proof_server_name, server_name, |span: &ErrorInfoSpan<'_>| span.server_name().bytes.len() <= span.bytes.len());
proof_accessor!(proof_proc_name, proc_name, |span: &ErrorInfoSpan<'_>| span.proc_name().bytes.len() <= span.bytes.len());
// proof_accessor!(proof_line_number, line_number, |span: &ErrorInfoSpan<'_>| span.line_number() == span.line_number());

#[cfg(kani)]
#[kani::proof_for_contract(ErrorInfoSpan::new)]
fn proof_error_info_for_contract() {
    let bytes: [u8; 128] = kani::any();
    let slice = kani::slice::any_slice_of_array(&bytes);
    let _ = ErrorInfoSpan::new(slice);
}