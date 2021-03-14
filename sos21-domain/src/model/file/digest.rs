use std::convert::TryInto;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FileBlake3Digest([u8; 32]);

#[derive(Debug, Clone, Error)]
#[error("invalid length of blake3 digest bytes")]
pub struct InvalidLengthError {
    _priv: (),
}

impl FileBlake3Digest {
    pub fn from_array(bytes: [u8; 32]) -> Self {
        FileBlake3Digest(bytes)
    }

    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, InvalidLengthError> {
        bytes
            .try_into()
            .map(FileBlake3Digest)
            .map_err(|_| InvalidLengthError { _priv: () })
    }

    pub fn into_array(self) -> [u8; 32] {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
}
