use crate::prelude::*;
use crate::tls::TlsHandshaker;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use rustls_pki_types::DnsName;
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::string::String;
use std::sync::Arc;
use webpki_roots;

#[derive(Debug)]
pub struct RustlsConfig {
    config: Arc<ClientConfig>,
}

impl Default for RustlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RustlsConfig {
    pub fn new() -> Self {
        let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = Arc::new(
            ClientConfig::builder_with_protocol_versions(&[
                #[cfg(feature = "tls1.3")]
                &rustls::version::TLS13,
                #[cfg(feature = "tls1.2")]
                &rustls::version::TLS12,
            ])
            .with_root_certificates(root_store)
            .with_no_client_auth(),
        );
        Self { config }
    }

    /// Trust the server certificate without verification.
    /// N.B. Analogous to checking the "Trust server certificate" box in the connection properties in SSMS.
    pub fn new_trust_server_certificate() -> Self {
        let config = Arc::new(
            ClientConfig::builder_with_protocol_versions(&[
                #[cfg(feature = "tls1.3")]
                &rustls::version::TLS13,
                #[cfg(feature = "tls1.2")]
                &rustls::version::TLS12,
            ])
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth(),
        );
        Self { config }
    }
}

impl TlsHandshaker for RustlsConfig {
    type Connection = ClientConnection;
    type HandshakeError = rustls::Error;

    async fn handshake<S: AsyncTransport>(
        &self,
        server_name: &str,
        stream: &mut S,
    ) -> Result<ClientConnection, rustls::Error>
    where
        S::Error: core::fmt::Debug,
    {
        let name = if let Ok(dns_name) = DnsName::try_from(String::from(server_name)) {
            ServerName::DnsName(dns_name)
        } else if let Ok(ip_addr) = server_name.parse::<IpAddr>() {
            ServerName::IpAddress(ip_addr.into())
        } else {
            return Err(rustls::Error::UnsupportedNameType);
        };

        let mut connection = ClientConnection::new(Arc::clone(&self.config), name)?;

        while connection.is_handshaking() {
            while connection.wants_write() {
                let mut tls_out = alloc::vec::Vec::new();
                connection
                    .write_tls(&mut tls_out)
                    .map_err(|e| rustls::Error::General(format!("write_tls: {}", e)))?;
                let mut offset = 0;
                while offset < tls_out.len() {
                    let n = stream
                        .write(&tls_out[offset..])
                        .await
                        .map_err(|e| {
                            rustls::Error::General(format!("transport write: {:?}", e))
                        })?;
                    if n == 0 {
                        return Err(rustls::Error::General("connection closed".into()));
                    }
                    offset += n;
                }
            }
            if !connection.is_handshaking() {
                break;
            }

            let mut scratch = [0u8; 4096];
            let n = stream
                .read(&mut scratch)
                .await
                .map_err(|e| rustls::Error::General(format!("transport read: {:?}", e)))?;
            if n == 0 {
                return Err(rustls::Error::General(
                    "connection closed during handshake".into(),
                ));
            }
            connection
                .read_tls(&mut &scratch[..n])
                .map_err(|e| rustls::Error::General(format!("read_tls: {}", e)))?;
            connection.process_new_packets()?;
        }

        Ok(connection)
    }
}

// RustlsProvider — wraps a transport + an established TLS connection

pub struct RustlsProvider<T: AsyncTransport> {
    transport: T,
    connection: ClientConnection,
}

impl<T: AsyncTransport> RustlsProvider<T> {
    pub fn from_parts(transport: T, connection: ClientConnection) -> Self {
        Self { transport, connection }
    }
}

impl<T: AsyncTransport> AsyncTransport for RustlsProvider<T>
where
    T::Error: core::fmt::Debug,
{
    type Error = std::io::Error;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        loop {
            match self.connection.reader().read(buf) {
                Ok(0) => return Ok(0),
                Ok(n) => return Ok(n),
                Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => return Err(e),
                _ => {}
            }

            let mut scratch = [0u8; 4096];
            let n = self
                .transport
                .read(&mut scratch)
                .await
                .map_err(|e| std::io::Error::other(format!("{:?}", e)))?;
            if n == 0 {
                return Ok(0);
            }
            self.connection.read_tls(&mut &scratch[..n])?;
            self.connection.process_new_packets().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e))
            })?;
        }
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let written = self.connection.writer().write(buf)?;
        while self.connection.wants_write() {
            let mut tls_out = alloc::vec::Vec::new();
            self.connection.write_tls(&mut tls_out)?;
            let mut offset = 0;
            while offset < tls_out.len() {
                let n = self
                    .transport
                    .write(&tls_out[offset..])
                    .await
                    .map_err(|e| std::io::Error::other(format!("{:?}", e)))?;
                if n == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "transport closed",
                    ));
                }
                offset += n;
            }
        }
        Ok(written)
    }

    fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<(), Self::Error> {
        self.transport
            .set_read_timeout(timeout)
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }

    fn set_write_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<(), Self::Error> {
        self.transport
            .set_write_timeout(timeout)
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }
}

// NoCertificateVerification

#[derive(Debug)]
pub struct NoCertificateVerification;
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::aws_lc_rs::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::aws_lc_rs::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
