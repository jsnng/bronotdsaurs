use crate::tds::prelude::*;
use alloc::borrow::ToOwned;

/// Implementation of [`Decode`] for [`AltMetaDataSpan`].
impl<'a> Decode<'a> for AltMetaDataSpan<'a> {
    type Owned = AltMetaDataToken;
    type Span = AltMetaDataSpan<'a>;
    type Error = DecodeError;
    fn populate(_buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        todo!()
        // Ok(AltMetaDataSpan::new(buf))
    }
    fn own(self) -> Self::Owned {
        todo!()
    }
}

/// Implementation of [`Decode`] for [`AltRowSpan`].
impl<'a> Decode<'a> for AltRowSpan<'a> {
    type Owned = AltRowToken;
    type Span = AltRowSpan<'a>;
    type Error = DecodeError;
    fn populate(_buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        todo!();
    }
    fn own(self) -> Self::Owned {
        todo!()
    }
}


/// Implementation of [`Decode`] for [`ColInfoSpan`].
impl<'a> Decode<'a> for ColInfoSpan<'a> {
    type Owned = ColInfoToken;
    type Span = ColInfoSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        ColInfoSpan::new(buf)
    }
     fn own(self) -> Self::Owned {
      let col_property = self.into_iter().map(|x| {
        ColPropertyBuilder::default()
            .col_num(x.col_num as u8)
            .table_num(x.table_num as u8)
            .status(x.status)
            .col_name(x.col_name.map(|s| s.to_string()))
            .build()
            .unwrap()
        }).collect();

        ColInfoTokenBuilder::default()
        .ty(self.ty().try_into().unwrap())
        .length(self.length())
        .col_property(col_property)
        .build()
        .unwrap()
}}

/// Implementation of [`Decode`] for [`DataClassificationSpan`].
#[cfg(feature = "tds7.4")]
impl<'a> Decode<'a> for DataClassificationSpan<'a> {
    type Owned = DataClassificationToken;
    type Span = DataClassificationSpan<'a>;
    type Error = DecodeError;
    fn populate(_buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        todo!();
    }
    fn own(self) -> Self::Owned {
        todo!()
    }
}

/// Implementation of [`Decode`] for [`DoneSpan`].
impl<'a> Decode<'a> for DoneSpan<'a> {
    type Owned = DoneToken;
    type Span = DoneSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        DoneSpan::new(buf)
    }
    #[inline]
    fn own(self) -> Self::Owned {
        DoneToken {
            ty: self.ty(),
            status: self.status(),
            current_cmd: self.current_cmd(),
            done_row_count: self.done_row_count(),
        }
    }
}

/// Implementation of [`Decode`] for [`EnvChangeSpan`].
impl<'a> Decode<'a> for EnvChangeSpan<'a> {
    type Owned = EnvChangeToken;
    type Span = EnvChangeSpan<'a>;
    type Error = DecodeError;

    #[inline]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        let span = EnvChangeSpan::new(buf)?;
        span.ty().unwrap();
        let ib_new_val = span.ib_new_value()?;
        let cch_new_val = span.cch_new_value()?;
        let ib_old_val = span.ib_old_value()?;
        let cch_old_val = span.cch_old_value()?;
        if buf.len() < ib_new_val + cch_new_val {
            return Err(DecodeError::InvalidField(format!(
                "EnvChangeSpan populate() buf.len()={} < ib_new_val+cch_new_val={}",
                buf.len(),
                ib_new_val + cch_new_val
            )));
        }
        if buf.len() < ib_old_val + cch_old_val {
            return Err(DecodeError::InvalidField(format!(
                "EnvChangeSpan populate() buf.len()={} < ib_old_val+cch_old_val={}",
                buf.len(),
                ib_old_val + cch_old_val
            )));
        }
        Ok(span)
    }

    fn own(self) -> EnvChangeToken {
        let ty = self.ty().unwrap();
        let ib_new_val = self.ib_new_value().unwrap();
        let cch_new_val = self.cch_new_value().unwrap();
        let ib_old_val = self.ib_old_value().unwrap();
        let cch_old_val = self.cch_old_value().unwrap();
        let new_val = &self.bytes[ib_new_val..ib_new_val + cch_new_val];
        let old_val = &self.bytes[ib_old_val..ib_old_val + cch_old_val];
        let env_value_data = match ty {
            #[allow(deprecated)]
            EnvChangeType::Database
            | EnvChangeType::Language
            | EnvChangeType::CharacterSet
            | EnvChangeType::PacketSize
            | EnvChangeType::UnicodeDataSortingLocalID
            | EnvChangeType::UnicodeDataSortingComparisonFlags => EnvValueData::BVarChar {
                new: NVarCharSpan::new(new_val).to_string(),
                old: NVarCharSpan::new(old_val).to_string(),
            },
            #[cfg(feature = "tds7.2")]
            EnvChangeType::RealTimeLogShipping | EnvChangeType::SendUserToClientRequest => {
                EnvValueData::BVarChar {
                    new: NVarCharSpan::new(new_val).to_string(),
                    old: NVarCharSpan::new(old_val).to_string(),
                }
            }
            EnvChangeType::SQLCollation => EnvValueData::BVarBytes {
                new: new_val.to_vec(),
                old: old_val.to_vec(),
            },
            #[cfg(feature = "tds7.2")]
            EnvChangeType::BeginTransaction
            | EnvChangeType::CommitTransaction
            | EnvChangeType::RollbackTransaction
            | EnvChangeType::EnlistDTCTransaction
            | EnvChangeType::DefectTransaction
            | EnvChangeType::TransactionEnded => EnvValueData::BVarBytes {
                new: new_val.to_vec(),
                old: old_val.to_vec(),
            },
            #[cfg(feature = "tds7.2")]
            #[allow(deprecated)]
            EnvChangeType::TransactionManagerAddress => EnvValueData::BVarBytes {
                new: new_val.to_vec(),
                old: old_val.to_vec(),
            },
            _ => unreachable!("validated by populate"),
        };
        EnvChangeToken {
            ty,
            length: self.length() as usize,
            env_value_data,
        }
    }
}

/// Implementation of [`Decode`] for [`ErrorInfoSpan`].
/// This is used for both [`ClientMessageType::Error`](crate::tds::types::tokens::types::DataTokenType::Error)
/// and [`DataTokenType::Info`](crate::tds::types::tokens::types::DataTokenType::Info).
impl<'a> Decode<'a> for ErrorInfoSpan<'a> {
    type Owned = ErrorInfoToken;
    type Error = DecodeError;
    type Span = ErrorInfoSpan<'a>;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        ErrorInfoSpan::new(buf)
    }

    #[inline]
    fn own(self) -> Self::Owned {
        ErrorInfoToken {
            ty: self.ty(),
            length: self.length(),
            number: self.number(),
            state: self.state(),
            class: ErrorClass::try_from(self.class()).unwrap(),
            msg_text: self.msg_text().to_string(),
            server_name: self.server_name().to_string(),
            proc_name: self.proc_name().to_string(),
            line_number: self.line_number(),
        }
    }
}

/// Implementation of [`Decode`] for [`LoginAckSpan`].
impl<'a> Decode<'a> for LoginAckSpan<'a> {
    type Owned = LoginAckToken;
    type Error = DecodeError;
    type Span = LoginAckSpan<'a>;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        LoginAckSpan::new(buf)
    }

    fn own(self) -> Self::Owned {
        LoginAckTokenBuilder::default()
            .ty(self.ty())
            .length(self.length())
            .interface(self.interface())
            .tds_version(self.tds_version())
            .prog_name(self.prog_name().to_string())
            .build()
            .unwrap()
    }
}

/// Implementation of [`Decode`] for [`OffsetSpan`].
#[cfg(all(feature = "tds7.1", not(feature = "tds7.2")))]
impl<'a> Decode<'a> for OffsetSpan<'a> {
    type Owned = OffsetToken;
    type Span = OffsetSpan<'a>;
    type Error = DecodeError;

    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        Ok(OffsetSpan { bytes: buf })
    }
    fn own(self) -> Self::Owned {
        OffsetToken {
            identifier: r_u16_le(self.bytes, 1),
            offset_len: r_u16_le(self.bytes, 3),
        }
    }
}

/// Implementation of [`Decode`] for [`OrderSpan`].
impl<'a> Decode<'a> for OrderSpan<'a> {
    type Owned = OrderToken;
    type Span = OrderSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        OrderSpan::new(buf)
    }
    fn own(self) -> Self::Owned {
        OrderToken {
            length: self.length(),
            col_num: self.into_iter().collect(),
        }
    }
}

/// Implementation of [`Decode`] for [`ReturnStatusSpan`].
impl<'a> Decode<'a> for ReturnStatusSpan<'a> {
    type Owned = ReturnStatusToken;
    type Span = ReturnStatusSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        ReturnStatusSpan::new(buf)
    }

    #[inline]
    fn own(self) -> Self::Owned {
        ReturnStatusToken { val: self.val() }
    }
}

/// Implementation of [`Decode`] for [`ReturnValueSpan`].
impl<'a> Decode<'a> for ReturnValueSpan<'a> {
    type Owned = ReturnValueToken;
    type Span = ReturnValueSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        ReturnValueSpan::new(buf)
    }
    fn own(self) -> Self::Owned {
        let param_name = self.param_name().bytes;
        let utf16: Vec<u16> = param_name
            .chunks_exact(2)
            .map(|b| r_u16_le( b, 0))
            .collect();
        ReturnValueTokenBuilder::default()
            .ty(self.ty())
            .param_ordinal(self.param_ordinal())
            .param_name(String::from_utf16_lossy(&utf16).to_owned())
            .status(self.status())
            .user_type(self.user_type())
            .flags(self.flags())
            .type_info(self.type_info().to_vec())
            .crypto_metadata(self.crypto_metadata().to_vec())
            .value(self.value().to_vec())
            .build()
            .unwrap()
    }
}

/// Implementation of [`Decode`] for [`SessionStatusSpan`].
#[cfg(feature = "tds7.4")]
impl<'a> Decode<'a> for SessionStatusSpan<'a> {
    type Owned = SessionStatusToken;
    type Span = SessionStatusSpan<'a>;
    type Error = DecodeError;

    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        SessionStatusSpan::new(buf)
    }

    fn own(self) -> Self::Owned {
        let dataset: Vec<SessionStatusData> = self.into_iter()
        .map(|x| SessionStatusData {
            state_id: x.state_id,
            state_value: x.state_value.to_vec()
        })
        .collect();

        SessionStatusToken {
            ty: self.ty(),
            length: self.length(),
            seq_no: self.seq_no(),
            status: self.status(),
            session_state_dataset: SessionStatusDataset(dataset),
        }
    }
}

/// Implementation of [`Decode`] for [`SspiSpan`].
impl<'a> Decode<'a> for SspiSpan<'a> {
    type Owned = SspiToken;
    type Error = DecodeError;
    type Span = SspiSpan<'a>;

    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        SspiSpan::new(buf)
    }

    fn own(self) -> Self::Owned {
        SspiToken {
            ty:  self.ty(),
            sspi_buffer: self.sspi_buffer().to_vec()
        }
    }
}

/// Implementation of [`Decode`] for [`TabNameSpan`].
impl<'a> Decode<'a> for TabNameSpan<'a> {
    type Owned = TabNameToken;
    type Span = TabNameSpan<'a>;
    type Error = DecodeError;

    #[inline(always)]
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        TabNameSpan::new(buf)
    }

    fn own(self) -> Self::Owned {
        let items: Vec<TabNameTokenItem> = self.into_iter().map(|x| TabNameTokenItem {
            parts: x.into_iter().map(|y| y.to_string()).collect(),
        }).collect();

        TabNameTokenBuilder::default()
        .ty(self.ty())
        .length(self.length())
        .items(items)
        .build()
        .unwrap()
    }
}

#[cfg(feature = "std")]
#[cfg(all(feature = "tds7.4", not(feature = "tds8.0")))]
#[cfg(test)]
mod tests {
    
    #[test]
    fn test_featureextack_with_azuresqlsupport_feature_data() {
        let _example: [u32; 707] = [
            0x04, 0x01, 0x02, 0xC3, 0x00, 0x77, 0x01, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0x01, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
            0x11, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x00, 0xC0, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0x11, 0x00, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x00,
            0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xFF, 0x11, 0x00, 0xC1, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE3, 0x1B,
            0x00, 0x01, 0x06, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x62, 0x00, 0x06,
            0x6D, 0x00, 0x61, 0x00, 0x73, 0x00, 0x74, 0x00, 0x65, 0x00, 0x72, 0x00, 0xAB, 0x66, 0x00, 0x45,
            0x16, 0x00, 0x00, 0x02, 0x00, 0x25, 0x00, 0x43, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67,
            0x00, 0x65, 0x00, 0x64, 0x00, 0x20, 0x00, 0x64, 0x00, 0x61, 0x00, 0x74, 0x00, 0x61, 0x00, 0x62,
            0x00, 0x61, 0x00, 0x73, 0x00, 0x65, 0x00, 0x20, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x74,
            0x00, 0x65, 0x00, 0x78, 0x00, 0x74, 0x00, 0x20, 0x00, 0x74, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x27,
            0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x62, 0x00, 0x27, 0x00, 0x2E,
            0x00, 0x07, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x73, 0x00, 0x76, 0x00, 0x72, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0xE3, 0x08, 0x00, 0x07, 0x05, 0x09, 0x04, 0xD0, 0x00, 0x34, 0x00,
            0xE3, 0x17, 0x00, 0x02, 0x0A, 0x75, 0x00, 0x73, 0x00, 0x5F, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x67,
            0x00, 0x6C, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68, 0x00, 0x00, 0xAB, 0x6A, 0x00, 0x47, 0x16, 0x00,
            0x00, 0x01, 0x00, 0x27, 0x00, 0x43, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x65,
            0x00, 0x64, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x75, 0x00, 0x61,
            0x00, 0x67, 0x00, 0x65, 0x00, 0x20, 0x00, 0x73, 0x00, 0x65, 0x00, 0x74, 0x00, 0x74, 0x00, 0x69,
            0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00, 0x74, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x75, 0x00, 0x73,
            0x00, 0x5F, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x6C, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68,
            0x00, 0x2E, 0x00, 0x07, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x73, 0x00, 0x76, 0x00,
            0x72, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xAD, 0x36, 0x00, 0x01, 0x74, 0x00, 0x00, 0x04, 0x16,
            0x4D, 0x00, 0x69, 0x00, 0x63, 0x00, 0x72, 0x00, 0x6F, 0x00, 0x73, 0x00, 0x6F, 0x00, 0x66, 0x00,
            0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x51, 0x00, 0x4C, 0x00, 0x20, 0x00, 0x53, 0x00, 0x65, 0x00,
            0x72, 0x00, 0x76, 0x00, 0x65, 0x00, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x03, 0xE8,
            0xE3, 0x13, 0x00, 0x04, 0x04, 0x38, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x04, 0x34, 0x00,
            0x30, 0x00, 0x39, 0x00, 0x36, 0x00, 0xAE, 0x01, 0x77, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x60,
            0x81, 0x14, 0xFF, 0xE7, 0xFF, 0xFF, 0x00, 0x02, 0x02, 0x07, 0x01, 0x04, 0x01, 0x00, 0x05, 0x04,
            0xFF, 0xFF, 0xFF, 0xFF, 0x06, 0x01, 0x00, 0x07, 0x01, 0x02, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x09, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0x0B, 0x47, 0x35, 0x00, 0x44, 0x00,
            0x37, 0x00, 0x45, 0x00, 0x44, 0x00, 0x37, 0x00, 0x30, 0x00, 0x42, 0x00, 0x2D, 0x00, 0x42, 0x00,
            0x39, 0x00, 0x32, 0x00, 0x45, 0x00, 0x2D, 0x00, 0x34, 0x00, 0x31, 0x00, 0x32, 0x00, 0x42, 0x00,
            0x2D, 0x00, 0x42, 0x00, 0x33, 0x00, 0x32, 0x00, 0x46, 0x00, 0x2D, 0x00, 0x37, 0x00, 0x36, 0x00,
            0x30, 0x00, 0x43, 0x00, 0x44, 0x00, 0x37, 0x00, 0x34, 0x00, 0x44, 0x00, 0x42, 0x00, 0x39, 0x00,
            0x32, 0x00, 0x43, 0x04, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08,
            0x01, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ];
    }
}