use crate::tds::prelude::*;
use crate::tds::session::prelude::*;

#[cfg(kani)]
use kani;

/// Writes `s` as UTF-16LE into `buf` starting at `ib`. Returns `(ib, cch)`.
fn write_utf16(buf: &mut [u8], ib: usize, s: &str) -> (usize, u16) {
    let mut cursor = ib;
    let mut chars = 0u16;
    for char in s.encode_utf16() {
        buf[cursor..][..size_of::<u16>()].copy_from_slice(&char.to_le_bytes());
        cursor += size_of::<u16>();
        chars += 1;
    }
    (cursor, chars)
}

fn write_tds_password_obfuscation(buf: &mut [u8], ib: usize, s: &str) -> (usize, u16) {
    let mut cursor = ib;
    let mut chars = 0u16;
    for char in s.encode_utf16() {
        let b = char.to_le_bytes();
        let obfuscated = [
            ((b[0] & 0x0f) << 4 | (b[0] >> 4)) ^ Login7Packet::PWD_XOR_MASK,
            ((b[1] & 0x0f) << 4 | (b[1] >> 4)) ^ Login7Packet::PWD_XOR_MASK,
        ];
        buf[cursor..][..size_of::<u16>()].copy_from_slice(&obfuscated);
        cursor += size_of::<u16>();
        chars += 1;
    }
    (cursor, chars)
}

#[inline]
fn write_ut16_optional(buf: &mut [u8], ib: &mut usize, field: &Option<String>) -> (u16, u16) {
    let ib_field = *ib as u16;
    let Some(s) = field else { return (ib_field, 0) };
    let (end, cch) = write_utf16(buf, *ib, s);
    *ib = end;
    (ib_field, cch)
}


impl MessageEncoder for Login7Packet {
    type Error = EncodeError;
    type Header = Login7Header;

    fn oneshot(
        &self,
        buf: &mut SessionBuffer,
        header: &mut Self::Header,
    ) -> Result<usize, Self::Error> {
        // ib - index of beginning
        // cch - count of characters (length in unicode characters)
        let mut ib = Login7Header::FIXED_DATA_OFFSET;

        let (ib_host_name, cch_host_name) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.host_name);

        let (ib_user_name, cch_user_name) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.user_name);

        let mut ib_password: u16 = 0;
        let mut cch_password: u16 = 0;
        if let Some(password) = &self.password {
            ib_password = ib as u16;
            let (end, count) = write_tds_password_obfuscation(buf.writeable(), ib, password);
            ib = end;
            cch_password = count;
        }

        let (ib_app_name, cch_app_name) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.app_name);
        let (ib_server_name, cch_server_name) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.server_name);

        #[cfg(feature = "tds7.4")]
        let mut ib_extension: u16 = 0;
        #[cfg(feature = "tds7.4")]
        let mut cch_extension: u16 = 0;
        //FEATUREEXT
        #[cfg(feature = "tds7.4")]
        let feature_ext_offset = if !self.feature_ext.is_empty() {
            ib_extension = ib as u16;
            cch_extension = size_of::<u32>() as u16;
            ib += size_of::<u32>();
            Some(ib_extension)
        } else {
            None
        };

        let (ib_clt_int_name, cch_clt_int_name) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.clt_int_name);
        let (ib_language, cch_language) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.language);
        let (ib_database, cch_database) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.database);
        let (ib_sspi, cch_sspi) = write_ut16_optional(buf.writeable(), &mut ib, &self.sspi);
        let (ib_atch_db_file, cch_atch_db_file) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.atch_db_file);
        #[cfg(feature = "tds7.2")]
        let (ib_change_password, cch_change_password) =
            write_ut16_optional(buf.writeable(), &mut ib, &self.change_password);

        #[cfg(feature = "tds7.4")]
        if let Some(offset) = feature_ext_offset {
            buf.writeable()[offset as usize..offset as usize + size_of::<u32>()]
                .copy_from_slice(&((ib - Login7Header::LENGTH) as u32).to_le_bytes());
            for feature in &self.feature_ext {
                ib += feature.encode(&mut buf.writeable()[ib..]);
            }
            buf.writeable()[ib] = FeatureExtType::Terminator as u8;
            ib += size_of::<u8>();
        }

        let ib = ib as u16;
        let relative_to_body = |abs: u16| abs - Login7Header::LENGTH as u16;
        let ib_or_default = |ib_: u16, _cch: u16| relative_to_body(ib_);
        let mut cursor: usize = 44;
        wint!(buf.writeable(), cursor, u16, relative_to_body(ib_host_name));
        wint!(buf.writeable(), cursor, u16, cch_host_name);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_user_name, cch_user_name));
        wint!(buf.writeable(), cursor, u16, cch_user_name);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_password, cch_password));
        wint!(buf.writeable(), cursor, u16, cch_password);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_app_name, cch_app_name));
        wint!(buf.writeable(), cursor, u16, cch_app_name);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_server_name, cch_server_name));
        wint!(buf.writeable(), cursor, u16, cch_server_name);
        // cursor is now at 64
        #[cfg(feature = "tds7.4")]
        {
            wint!(buf.writeable(), cursor, u16, if ib_extension > 0 { relative_to_body(ib_extension) } else { 0 });
            wint!(buf.writeable(), cursor, u16, cch_extension);
        }
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_clt_int_name, cch_clt_int_name));
        wint!(buf.writeable(), cursor, u16, cch_clt_int_name);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_language, cch_language));
        wint!(buf.writeable(), cursor, u16, cch_language);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_database, cch_database));
        wint!(buf.writeable(), cursor, u16, cch_database);
        buf.writeable()[cursor..cursor + size_of::<[u8; 6]>()].copy_from_slice(&self.client_id.unwrap_or([0u8; 6]));
        cursor += size_of::<[u8; 6]>();
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_sspi, cch_sspi));
        wint!(buf.writeable(), cursor, u16, cch_sspi);
        wint!(buf.writeable(), cursor, u16, ib_or_default(ib_atch_db_file, cch_atch_db_file));
        wint!(buf.writeable(), cursor, u16, cch_atch_db_file);
        #[cfg(feature = "tds7.2")]
        {
            wint!(buf.writeable(), cursor, u16, ib_or_default(ib_change_password, cch_change_password));
            wint!(buf.writeable(), cursor, u16, cch_change_password);
        }
        buf.writeable()[cursor..cursor + size_of::<u32>()].copy_from_slice(&0u32.to_le_bytes());
        let mut cursor = 8usize;
        wint!(buf.writeable(), cursor, u32, (ib - Login7Header::LENGTH as u16) as u32);
        wint!(buf.writeable(), cursor, u32, self.tds_version);
        wint!(buf.writeable(), cursor, u32, self.packet_size);
        wint!(buf.writeable(), cursor, u32, self.client_prog_ver);
        wint!(buf.writeable(), cursor, u32, self.client_pid);
        buf.writeable()[cursor..cursor + size_of::<u32>()].copy_from_slice(&self.connection_id.to_le_bytes());
        buf.writeable()[32] = self.option_flag1.0;
        buf.writeable()[33] = self.option_flag2.0;
        buf.writeable()[34] = self.type_flags.0;
        #[cfg(feature = "tds7.4")]
        {
            buf.writeable()[35] = self.option_flag3.0;
        }
        let mut cursor = 36usize;
        #[cfg(feature = "tds7.2")]
        wint!(buf.writeable(), cursor, u32, self.client_time_zone);
        #[cfg(not(feature = "tds7.2"))]
        buf.writeable()[cursor..cursor + size_of::<u32>()].copy_from_slice(&self.client_time_zone.to_le_bytes());
        #[allow(deprecated)]
        #[cfg(feature = "tds7.2")]
        buf.writeable()[cursor..cursor + size_of::<u32>()].copy_from_slice(&self.client_lc_id.to_le_bytes());
        header.length = ib;
        buf.writeable()[..Login7Header::LENGTH].copy_from_slice(&header.as_bytes());

        Ok(ib.into())
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for FeatureOption {
    fn encode(&self, buf: &mut [u8]) -> usize {
        let length = match self {
            Self::Terminator => {
                buf[0] = FeatureExtType::Terminator as u8;
                return 1;
            }
            Self::SessionRecovery(val) => {
                buf[0] = FeatureExtType::SessionRecovery as u8;
                val.encode(&mut buf[5..])
            }
            Self::FedAuth(val) => {
                buf[0] = FeatureExtType::FedAuth as u8;
                val.encode(&mut buf[5..])
            }
            Self::ColumnEncryption(val) => {
                buf[0] = FeatureExtType::ColumnEncryption as u8;
                val.encode(&mut buf[5..])
            }
            Self::GlobalTransactions => {
                // NO DATA
                buf[0] = FeatureExtType::GlobalTransactions as u8;
                0
            }
            Self::AzureSQLSupport(val) => {
                buf[0] = FeatureExtType::AzureSQLSupport as u8;
                val.encode(&mut buf[5..])
            }
            Self::DataClassification(val) => {
                buf[0] = FeatureExtType::DataClassification as u8;
                val.encode(&mut buf[5..])
            }
            Self::UTF8Support(val) => {
                buf[0] = FeatureExtType::UTF8Support as u8;
                val.encode(&mut buf[5..])
            }
            Self::AzureSQLDNSCaching => {
                // NO DATA
                buf[0] = FeatureExtType::AzureSQLDNSCaching as u8;
                0
            }
            Self::JsonSupport(val) => {
                buf[0] = FeatureExtType::JsonSupport as u8;
                val.encode(&mut buf[5..])
            }
            Self::VectorSupport(val) => {
                buf[0] = FeatureExtType::VectorSupport as u8;
                val.encode(&mut buf[5..])
            }
            Self::EnhancedRoutingSupport => {
                // NO DATA
                buf[0] = FeatureExtType::EnhancedRoutingSupport as u8;
                0
            }
            _ => todo!(),
        };
        buf[1..1+size_of::<u32>()].copy_from_slice(&(length as u32).to_le_bytes());
        FeatureOption::LENGTH + length
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for SessionRecoveryData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        if self.length == 0 &&
            self.recovery_database.is_empty() &&
            self.recovery_language.is_empty() {
                return 0;
        }
        let mut cursor: usize = 0;
        buf[cursor] = 1;
        cursor += 2;
        self.recovery_database.encode_utf16().for_each(|x| { wint!(buf, cursor, u16, x); });
        buf[1] = ((cursor - 2) / 2) as u8; 
        cursor += self.recovery_collation.encode(&mut buf[cursor..]);
        let idx = cursor;
        cursor += 1;
        self.recovery_language.encode_utf16().for_each(|x| { wint!(buf, cursor, u16, x); });
        buf[idx] = ((cursor - idx - 1) / 2) as u8;
        wint!(buf, cursor, u32, self.length);
        cursor
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for FedAuthKind {
    fn encode(&self, _buf: &mut [u8]) -> usize {
        match self {
            #[cfg(feature = "fed-auth-adal")]
            Self::Adal(val) => val.encode(_buf),
            #[cfg(feature = "fed-auth-token")]
            Self::SecurityToken(val) => val.encode(_buf),
            _ => unreachable!()
        }
    }
}

#[cfg(all(feature = "tds7.4", feature = "fed-auth-adal"))]
impl Encoder for FedAuthAdal {
    fn encode(&self, buf: &mut [u8]) -> usize {
        // b_fed_auth_library = 2 (ADAL), f_fed_auth_echo in high bit
        buf[0] = 0x02 | ((self.echo as u8) << 7);
        buf[1] = self.workflow as u8;
        2
    }
}

#[cfg(all(feature = "tds7.4", feature = "fed-auth-token"))]
impl Encoder for FedAuthSecurityToken {
    fn encode(&self, buf: &mut [u8]) -> usize {
        // [b_fed_auth_library: 7 bits | f_fed_auth_echo: 1 bit]
        // b_fed_auth_library = 1 (security token), f_fed_auth_echo in high bit
        buf[0] = 0x01 | ((self.echo as u8) << 7);
        let mut cursor = 1;
        let token = self.fed_auth_token.as_bytes();
        buf[cursor..cursor + 4].copy_from_slice(&(token.len() as u32).to_le_bytes());
        cursor += 4;
        buf[cursor..cursor + token.len()].copy_from_slice(token);
        cursor += token.len();
        buf[cursor..cursor + 32].copy_from_slice(&self.nonce);
        cursor += 32;
        buf[cursor..cursor + 4].copy_from_slice(&(self.channel_binding_token.len() as u32).to_le_bytes());
        cursor += 4;
        buf[cursor..cursor + self.channel_binding_token.len()].copy_from_slice(&self.channel_binding_token);
        cursor += self.channel_binding_token.len();
        buf[cursor..cursor + 32].copy_from_slice(&self.signature);
        cursor += 32;
        cursor
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for ColumnEncryptionData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.version as u8;
        1
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for AzureSQLSupportData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.0 as u8;
        1
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for DataClassificationData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self._version as u8;
        1
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for UTF8SupportData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.0 as u8;
        1
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for JsonSupportData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self._version;
        1
    }
}

#[cfg(feature = "tds7.4")]
impl Encoder for VectorSupportData {
    fn encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self._version;
        1
    }
}

#[cfg(feature = "std")]
#[cfg(all(feature = "tds7.4", not(feature = "tds8.0")))]
#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_login_encode() {
        let capture: [u8; 290] = [
            0x10, 0x01, 0x01, 0x22, 0x00, 0x00, 0x00, 0x00, 0x1a, 0x01, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x74, 0x00, 0x10, 0x00, 0x00, 0x06, 0x83, 0xf2, 0xf8, 0xe8, 0xf0, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xe0, 0x01, 0x00, 0x08, 0x88, 0xff, 0xff, 0xff, 0x36, 0x04,
            0x00, 0x00, 0x5e, 0x00, 0x03, 0x00, 0x64, 0x00, 0x08, 0x00, 0x74, 0x00, 0x10, 0x00,
            0x94, 0x00, 0x21, 0x00, 0xd6, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf2, 0x00,
            0x0a, 0x00, 0x06, 0x01, 0x0a, 0x00, 0x1a, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x1a, 0x01, 0x00, 0x00, 0x1a, 0x01, 0x00, 0x00, 0x1a, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x4d, 0x00, 0x61, 0x00, 0x63, 0x00, 0x56, 0x00, 0x4e, 0x00,
            0x53, 0x00, 0x57, 0x00, 0x47, 0x00, 0x65, 0x00, 0x6d, 0x00, 0x61, 0x00, 0x22, 0xa5,
            0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5,
            0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5, 0x22, 0xa5,
            0x22, 0xa5, 0x53, 0x00, 0x51, 0x00, 0x4c, 0x00, 0x50, 0x00, 0x72, 0x00, 0x6f, 0x00,
            0x20, 0x00, 0x66, 0x00, 0x6f, 0x00, 0x72, 0x00, 0x20, 0x00, 0x4d, 0x00, 0x53, 0x00,
            0x53, 0x00, 0x51, 0x00, 0x4c, 0x00, 0x20, 0x00, 0x28, 0x00, 0x68, 0x00, 0x61, 0x00,
            0x6e, 0x00, 0x6b, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x73, 0x00, 0x6f, 0x00, 0x66, 0x00,
            0x74, 0x00, 0x2e, 0x00, 0x63, 0x00, 0x6f, 0x00, 0x6d, 0x00, 0x29, 0x00, 0x31, 0x00,
            0x30, 0x00, 0x33, 0x00, 0x2e, 0x00, 0x38, 0x00, 0x36, 0x00, 0x2e, 0x00, 0x31, 0x00,
            0x33, 0x00, 0x36, 0x00, 0x2e, 0x00, 0x31, 0x00, 0x33, 0x00, 0x39, 0x00, 0x44, 0x00,
            0x42, 0x00, 0x2d, 0x00, 0x4c, 0x00, 0x69, 0x00, 0x62, 0x00, 0x72, 0x00, 0x61, 0x00,
            0x72, 0x00, 0x79, 0x00, 0x75, 0x00, 0x73, 0x00, 0x5f, 0x00, 0x65, 0x00, 0x6e, 0x00,
            0x67, 0x00, 0x6c, 0x00, 0x69, 0x00, 0x73, 0x00, 0x68, 0x00,
        ];

        let login7 = Login7PacketBuilder::default()
            .host_name("Mac".to_string())
            .user_name("VNSWGema".to_string())
            .password("xxxxxxxxxxxxxxxx".to_string())
            .clt_int_name("DB-Library".to_string())
            .app_name("SQLPro for MSSQL (hankinsoft.com)".to_string())
            .server_name("103.86.136.139".to_string())
            .language("us_english".to_string())
            .tds_version(0x74000004u32)
            .packet_size(4096u32)
            .client_prog_ver(0xf8f28306u32)
            .client_pid(70768u32)
            .option_flag1(OptionFlag1(0xe0))
            .option_flag2(OptionFlag2(0x01))
            .client_time_zone(0xffffff88u32)
            .client_lc_id(0x00000436u32)
            .option_flag3(OptionFlag3(0x08))
            .build()
            .unwrap();

        let mut buffer = SessionBuffer::default();

        let n = login7
            .oneshot(&mut buffer, &mut Login7Header::default())
            .expect("login7.encode() failed?");
        // assert_eq!(login7.header().as_ref().unwrap_or(&Login7Header::default()).length, n as u16);
        let _n = buffer.tail(n);
        let readable = &buffer.readable()[..n];

        for (i, (c_chunk, e_chunk)) in capture.chunks(16).zip(readable.chunks(16)).enumerate() {
            let c_hex: String = c_chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let e_hex: String = e_chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let marker = if c_hex == e_hex { " " } else { "!" };
            println!("{:03}: {:<48} {} {:<48}", i * 16, c_hex, marker, e_hex);
            // assert_eq!(c_hex, e_hex);
        }

        let bytes: &[u8; 8] = buffer.readable()[..8].try_into().unwrap();
        println!("{:?}", bytes);

        let header = Login7Header::from_bytes(bytes);
        println!("{}", header);
        assert_eq!(header.ty, ClientMessageType::TDS7Login);
        assert_eq!(header.length, n as u16);

        let bytes: &[u8; 8] = &capture[..8].try_into().unwrap_or([0u8; 8]);
        let example = Login7Header::from_bytes(bytes);
        assert_eq!(example.ty, ClientMessageType::TDS7Login);
        assert_eq!(example.length, n as u16);
        // println!("{}", buffer);
    }

    #[test]
    fn test_featureext_with_azuresqlsupport_feature_data() {
    let example: [u8; 455] = [
        0x10, 0x01, 0x01, 0xC7, 0x00, 0x00, 0x01, 0x00, 0xBF, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x74,
        0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x96, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xE0, 0x03, 0x20, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5E, 0x00, 0x0C, 0x00,
        0x76, 0x00, 0x07, 0x00, 0x84, 0x00, 0x08, 0x00, 0x94, 0x00, 0x1C, 0x00, 0xCC, 0x00, 0x4A, 0x00,
        0x60, 0x01, 0x04, 0x00, 0x64, 0x01, 0x1C, 0x00, 0x9C, 0x01, 0x00, 0x00, 0x9C, 0x01, 0x06, 0x00,
        0xC2, 0xCC, 0x3D, 0x20, 0xB7, 0xAB, 0xAB, 0x01, 0x00, 0x00, 0xAB, 0x01, 0x00, 0x00, 0xAB, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5A, 0x00, 0x4C, 0x00, 0x49, 0x00, 0x4E, 0x00, 0x36, 0x00,
        0x43, 0x00, 0x4C, 0x00, 0x49, 0x00, 0x45, 0x00, 0x4E, 0x00, 0x54, 0x00, 0x32, 0x00, 0x63, 0x00,
        0x6C, 0x00, 0x6F, 0x00, 0x75, 0x00, 0x64, 0x00, 0x73, 0x00, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x4E, 0x00,
        0x65, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x71, 0x00, 0x6C, 0x00, 0x43, 0x00, 0x6C, 0x00,
        0x69, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x20, 0x00, 0x44, 0x00, 0x61, 0x00, 0x74, 0x00,
        0x61, 0x00, 0x20, 0x00, 0x50, 0x00, 0x72, 0x00, 0x6F, 0x00, 0x76, 0x00, 0x69, 0x00, 0x64, 0x00,
        0x65, 0x00, 0x72, 0x00, 0x65, 0x00, 0x32, 0x00, 0x66, 0x00, 0x38, 0x00, 0x38, 0x00, 0x37, 0x00,
        0x36, 0x00, 0x61, 0x00, 0x64, 0x00, 0x36, 0x00, 0x35, 0x00, 0x38, 0x00, 0x2E, 0x00, 0x6C, 0x00,
        0x6F, 0x00, 0x63, 0x00, 0x61, 0x00, 0x6C, 0x00, 0x2E, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x65, 0x00,
        0x62, 0x00, 0x6F, 0x00, 0x78, 0x00, 0x2E, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x74, 0x00,
        0x72, 0x00, 0x6F, 0x00, 0x6C, 0x00, 0x2E, 0x00, 0x7A, 0x00, 0x6C, 0x00, 0x69, 0x00, 0x6E, 0x00,
        0x68, 0x00, 0x65, 0x00, 0x6B, 0x00, 0x61, 0x00, 0x36, 0x00, 0x64, 0x00, 0x65, 0x00, 0x76, 0x00,
        0x34, 0x00, 0x2E, 0x00, 0x6F, 0x00, 0x6E, 0x00, 0x65, 0x00, 0x62, 0x00, 0x6F, 0x00, 0x78, 0x00,
        0x2E, 0x00, 0x78, 0x00, 0x64, 0x00, 0x62, 0x00, 0x2E, 0x00, 0x6D, 0x00, 0x73, 0x00, 0x63, 0x00,
        0x64, 0x00, 0x73, 0x00, 0x2E, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x6D, 0x00, 0x2C, 0x00, 0x33, 0x00,
        0x37, 0x00, 0x30, 0x00, 0x30, 0x00, 0x38, 0x00, 0xA8, 0x01, 0x00, 0x00, 0x2E, 0x00, 0x4E, 0x00,
        0x65, 0x00, 0x74, 0x00, 0x20, 0x00, 0x53, 0x00, 0x71, 0x00, 0x6C, 0x00, 0x43, 0x00, 0x6C, 0x00,
        0x69, 0x00, 0x65, 0x00, 0x6E, 0x00, 0x74, 0x00, 0x20, 0x00, 0x44, 0x00, 0x61, 0x00, 0x74, 0x00,
        0x61, 0x00, 0x20, 0x00, 0x50, 0x00, 0x72, 0x00, 0x6F, 0x00, 0x76, 0x00, 0x69, 0x00, 0x64, 0x00,
        0x65, 0x00, 0x72, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x62, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x08, 0x01, 0x00, 0x00, 0x00, 0x01, 0xFF,
        ];

        let featureext: Vec<FeatureOption> = vec![
            FeatureOption::SessionRecovery(SessionRecoveryData { length: 0, recovery_database: String::new(), recovery_collation: Collation::default(), recovery_language: String::new() }),
            FeatureOption::ColumnEncryption(ColumnEncryptionData { version: ColumnEncryptionVersion::WithoutEnclaveComputations}),
            FeatureOption::GlobalTransactions,
            FeatureOption::AzureSQLSupport(AzureSQLSupportData(true)),
        ];

        let login7 = Login7PacketBuilder::default()
            .host_name("ZLIN6CLIENT2".to_string())
            .user_name("cloudsa".to_string())
            .password("\0\0\0\0\0\0\0\0".to_string())
            .database("testdb".to_string())
            .clt_int_name(".Net SqlClient Data Provider".to_string())
            .app_name(".Net SqlClient Data Provider".to_string())
            .server_name("e2f8876ad658.local.onebox.control.zlinheka6dev4.onebox.xdb.mscds.com,37008".to_string())
            .language("".to_string())
            .tds_version(0x74000004u32)
            .packet_size(8000u32)
            .client_prog_ver(0x06000000u32)
            .client_pid(0x00001d96u32)
            .client_id([0xc2, 0xcc, 0x3d, 0x20, 0xb7, 0xab])
            .option_flag1(OptionFlag1(0xe0))
            .option_flag2(OptionFlag2(0x03))
            .type_flags(TypeFlag(0x20))
            .client_time_zone(0x00000000u32)
            .client_lc_id(0x00000000u32)
            .option_flag3(OptionFlag3(0x10))
            .feature_ext(featureext)
            .build()
            .unwrap();

        let mut buffer = SessionBuffer::default();
        let n = login7
            .oneshot(&mut buffer, &mut Login7HeaderBuilder::default()
            .status(0x01)
            .packet_id(0x01)
            .build()
            .unwrap()
            )
            .expect("login7.encode() failed?");

        let _n = buffer.tail(n);
        let readable = &buffer.readable()[..n];

        for (i, (c_chunk, e_chunk)) in example.chunks(16).zip(readable.chunks(16)).enumerate() {
            let c_hex: String = c_chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let e_hex: String = e_chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let marker = if c_hex == e_hex { " " } else { "!" };
            println!("{:03}: {:<48} {} {:<48}", i * 16, c_hex, marker, e_hex);
            // assert_eq!(c_hex, e_hex);
        }

        let bytes: &[u8; 8] = buffer.readable()[..8].try_into().unwrap();
        println!("{:?}", bytes);

        let header = Login7Header::from_bytes(bytes);
        println!("{}", header);
        assert_eq!(header.ty, ClientMessageType::TDS7Login);
        assert_eq!(header.length, n as u16);

        let bytes: &[u8; 8] = &example[..8].try_into().unwrap_or([0u8; 8]);
        let example = Login7Header::from_bytes(bytes);
        assert_eq!(example.ty, ClientMessageType::TDS7Login);
        assert_eq!(example.length, n as u16);
        // println!("{}", buffer);
    }
}
