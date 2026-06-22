use crate::tds::prelude::*;

/// Implementation of [`Decode`] for [`PreLogin`](crate::tds::types::prelude::PreLoginSpan).
impl<'a> Decode<'a> for PreLoginSpan<'a> {
    type Owned = PreLoginPacket;
    type Error = DecodeError;
    type Span = PreLoginSpan<'a>;
    fn populate(buf: &'a [u8]) -> Result<Self::Span, Self::Error> {
        let mut span = PreLoginSpan::new(buf)?;
        for opt in span.options() {
            let token = opt.option_token();
            if token == Some(PLOptionType::Terminator) {
                break;
            }

            #[cfg(feature = "unsafe")]
            let offset = unsafe { opt.offset() }.ok_or(DecodeError::InvalidField(
                "PreLoginSpan populate() opt.offset is None".to_string(),
            ))? as usize;
            #[cfg(not(feature = "unsafe"))]
            let offset = opt.offset().ok_or(DecodeError::InvalidField(
                "PreLoginSpan populate() opt.offset is None".to_string(),
            ))? as usize;
            #[cfg(feature = "unsafe")]
            let len = unsafe { opt.option_length() }.ok_or(DecodeError::InvalidField(
                "PreLoginSpan populate() opt.option_length is None".to_string(),
            ))? as usize;
            #[cfg(not(feature = "unsafe"))]
            let len = opt.option_length().ok_or(DecodeError::InvalidField(
                "PreLoginSpan populate() opt.option_length is None".to_string(),
            ))? as usize;

            let cursor = PreLoginHeader::LENGTH + offset;

            if buf.len() < offset + len {
                return Err(DecodeError::UnexpectedEof(format!(
                    "PreLoginSpan populate() buf.len()={} < offset+len={}",
                    buf.len(),
                    offset + len
                )));
            }

            match token {
                Some(PLOptionType::Version) if len == 6 => {
                    span.version = Some(&buf[cursor..cursor + 6]);
                }
                Some(PLOptionType::Encryption) if len == 1 => {
                    span.encryption = Some(&buf[cursor..cursor + 1]);
                }
                Some(PLOptionType::InstOpt) => {
                    span.inst_opt = Some(&buf[cursor..cursor + 1]);
                }
                Some(PLOptionType::ThreadId) if len == 4 => {
                    span.thread_id = Some(&buf[cursor..cursor + 4]);
                }
                #[cfg(feature = "smp")]
                Some(PLOptionType::Mars) if len == 1 => {
                    span.mars(buf[cursor]);
                }
                #[cfg(not(feature = "smp"))]
                Some(PLOptionType::Mars) => {}
                Some(PLOptionType::Version | PLOptionType::Encryption | PLOptionType::ThreadId) => {
                }
                Some(PLOptionType::Terminator) => unreachable!(),
                _ => {
                    return Err(DecodeError::InvalidField(format!(
                        "PreLoginSpan populate() unknown option token: {:?}",
                        token
                    )));
                }
            }
        }

        Ok(span)
    }

    fn own(self) -> Self::Owned {
        let version = self.version();
        let mut builder = PreLoginPacketBuilder::default();
        builder.version(version);
        if let Some(e) = self.encryption() {
            builder.encryption(e);
        }
        if let Some(i) = self.inst_opt() {
            builder.inst_opt(i.to_vec());
        }
        if let Some(t) = self.thread_id() {
            builder.thread_id(t);
        }
        if let Some(m) = self.mars() {
            builder.mars(m);
        }
        builder.payload(self.payload().to_vec());
        builder.build().unwrap()
    }
}