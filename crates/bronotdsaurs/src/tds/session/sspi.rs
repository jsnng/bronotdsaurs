#[cfg(any(feature = "sspi", feature = "fedauth"))]
use crate::tds::prelude::*;

#[cfg(feature = "sspi")]
pub trait SspiProvider {
    type Error: core::fmt::Debug;
    fn next(&mut self, challenge: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

#[cfg(feature = "fedauth")]
pub trait FedAuthProvider {
    type Error: core::fmt::Debug;
    fn token(&mut self, fedauth_info: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
