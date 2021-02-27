use std::collections::HashSet;

use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::{self, LengthLimitedSet};
use crate::model::form::item::checkbox::CheckboxId;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct FormAnswerItemChecks(LengthLimitedSet<Unbounded, Bounded<typenum::U32>, CheckboxId>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromCheckedIdsErrorKind {
    TooLong,
    Duplicated(CheckboxId),
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer item checks")]
pub struct FromCheckedIdsError {
    kind: FromCheckedIdsErrorKind,
}

impl FromCheckedIdsError {
    pub fn kind(&self) -> FromCheckedIdsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError<Unbounded, Bounded<typenum::U32>>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromCheckedIdsErrorKind::TooLong,
            // TODO: statically assert unreachability
            collection::LengthErrorKind::TooShort => unreachable!(),
        };
        FromCheckedIdsError { kind }
    }
}

impl FormAnswerItemChecks {
    pub fn from_checked_ids<I>(checked_ids: I) -> Result<Self, FromCheckedIdsError>
    where
        I: IntoIterator<Item = CheckboxId>,
    {
        let mut result = LengthLimitedSet::new(HashSet::new()).unwrap();
        for checked_id in checked_ids {
            let has_inserted = result
                .insert(checked_id)
                .map_err(FromCheckedIdsError::from_length_error)?;
            if !has_inserted {
                return Err(FromCheckedIdsError {
                    kind: FromCheckedIdsErrorKind::Duplicated(checked_id),
                });
            }
        }

        Ok(FormAnswerItemChecks(result))
    }

    pub fn is_checked(&self, checkbox_id: CheckboxId) -> bool {
        self.0.contains(&checkbox_id)
    }

    pub fn count_checks(&self) -> usize {
        self.0.len()
    }

    pub fn checked_ids(&self) -> impl Iterator<Item = CheckboxId> + '_ {
        self.0.iter().copied()
    }
}

impl<'de> Deserialize<'de> for FormAnswerItemChecks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormAnswerItemChecks::from_checked_ids(Vec::<CheckboxId>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}
