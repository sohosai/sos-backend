use std::convert::TryInto;

use crate::model::bound::{Bounded, Unbounded};
use crate::model::integer::{self, LimitedInteger};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectIndex(LimitedInteger<Unbounded, Bounded<typenum::U999>, u16>);

#[derive(Debug, Error, Clone)]
#[error("invalid project index")]
pub struct FromU16Error {
    _priv: (),
}

impl FromU16Error {
    fn from_integer_error(_err: integer::BoundError) -> Self {
        FromU16Error { _priv: () }
    }
}

impl ProjectIndex {
    pub fn from_u16(index: u16) -> Result<ProjectIndex, FromU16Error> {
        let index = LimitedInteger::new(index).map_err(FromU16Error::from_integer_error)?;
        Ok(ProjectIndex(index))
    }

    pub fn to_u16(&self) -> u16 {
        self.0.into_inner()
    }

    pub fn to_i16(&self) -> i16 {
        // Since the index is 0 ..= 999, we can safely convert this to i16
        self.0.into_inner().try_into().unwrap()
    }
}
