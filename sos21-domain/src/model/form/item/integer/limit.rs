use crate::model::bound::{Bounded, Unbounded};
use crate::model::integer::LimitedInteger;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegerFormItemLimit(LimitedInteger<Bounded<typenum::U1>, Unbounded, u64>);

#[derive(Debug, Error, Clone)]
#[error("invalid integer form item limit")]
pub struct LimitError {
    _priv: (),
}

impl IntegerFormItemLimit {
    pub fn from_u64(limit: u64) -> Result<Self, LimitError> {
        let inner = LimitedInteger::new(limit).map_err(|_| LimitError { _priv: () })?;
        Ok(IntegerFormItemLimit(inner))
    }

    pub fn to_u64(self) -> u64 {
        self.0.into_inner()
    }
}
