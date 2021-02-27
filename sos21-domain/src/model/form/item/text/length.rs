use crate::model::integer::BoundedInteger;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextFormItemLength(BoundedInteger<typenum::U1, typenum::U1024, u64>);

#[derive(Debug, Error, Clone)]
#[error("invalid text form item length")]
pub struct LengthError {
    _priv: (),
}

impl TextFormItemLength {
    pub fn from_u64(length: u64) -> Result<Self, LengthError> {
        let inner = BoundedInteger::new(length).map_err(|_| LengthError { _priv: () })?;
        Ok(TextFormItemLength(inner))
    }

    pub fn to_u64(self) -> u64 {
        self.0.into_inner()
    }
}
