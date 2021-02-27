use crate::model::integer::BoundedInteger;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckboxFormItemLimit(BoundedInteger<typenum::U1, typenum::U31, u64>);

#[derive(Debug, Error, Clone)]
#[error("invalid checkbox form item limit")]
pub struct LimitError {
    _priv: (),
}

impl CheckboxFormItemLimit {
    pub fn from_u64(limit: u64) -> Result<Self, LimitError> {
        let inner = BoundedInteger::new(limit).map_err(|_| LimitError { _priv: () })?;
        Ok(CheckboxFormItemLimit(inner))
    }

    pub fn to_u64(self) -> u64 {
        self.0.into_inner()
    }
}
