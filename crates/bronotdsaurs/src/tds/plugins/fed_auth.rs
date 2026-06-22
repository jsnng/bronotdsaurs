#[cfg(feature = "fedauth")]
pub struct FedAuthBytes {
    pub fed_auth_token: Vec<u8>,
    pub nonce: Option<[u8; 32]>,
}

#[cfg(feature = "fedauth")]
pub trait FederatedAuthenticationPlugin {
    fn acquire(&self) -> Option<FedAuthBytes>;
    fn nonce(&self) -> Option<[u8; 32]>;
    fn sts_url(&self) -> impl ToString;
    fn spn(&self) -> impl ToString;
}

#[cfg(feature = "fedauth")]
impl FederatedAuthenticationPlugin for fedauth::fed_auth::FedAuthPlugin {
    fn acquire(&self) -> Option<FedAuthBytes> {
        let token = self.acquire().ok()?;
        Some(FedAuthBytes { fed_auth_token: token, nonce: self.nonce })
    }
    fn nonce(&self) -> Option<[u8; 32]> {
        self.nonce
    }
    fn sts_url(&self) -> impl ToString {
        self.sts_url.clone()
    }
    fn spn(&self) -> impl ToString {
        self.spn.clone()
    }
}
