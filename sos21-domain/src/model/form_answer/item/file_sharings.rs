use std::collections::HashSet;
use std::convert::TryInto;

use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::{self, LengthLimitedVec};
use crate::model::file::FileType;
use crate::model::file_sharing::FileSharingId;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct FormAnswerItemFileSharings(
    LengthLimitedVec<Unbounded, Bounded<typenum::U32>, FileSharingAnswer>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromSharingsErrorKind {
    Duplicated(FileSharingId),
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid file form answer item file sharing set")]
pub struct FromSharingsError {
    kind: FromSharingsErrorKind,
}

impl FromSharingsError {
    pub fn kind(&self) -> FromSharingsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError<Unbounded, Bounded<typenum::U32>>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromSharingsErrorKind::TooLong,
            // TODO: statically assert unreachability
            collection::LengthErrorKind::TooShort => unreachable!(),
        };
        FromSharingsError { kind }
    }
}

impl FormAnswerItemFileSharings {
    pub fn from_sharing_answers<I>(sharings: I) -> Result<Self, FromSharingsError>
    where
        I: IntoIterator<Item = FileSharingAnswer>,
    {
        let sharings = LengthLimitedVec::new(sharings.into_iter().collect())
            .map_err(FromSharingsError::from_length_error)?;

        let mut known_ids = HashSet::new();
        for answer in sharings.iter() {
            if !known_ids.insert(answer.sharing_id) {
                return Err(FromSharingsError {
                    kind: FromSharingsErrorKind::Duplicated(answer.sharing_id),
                });
            }
        }

        Ok(FormAnswerItemFileSharings(sharings))
    }

    pub fn sharing_answers(&self) -> impl Iterator<Item = &'_ FileSharingAnswer> {
        self.0.iter()
    }

    pub fn into_sharing_answers(self) -> impl Iterator<Item = FileSharingAnswer> {
        self.0.into_inner().into_iter()
    }

    pub fn len(&self) -> u8 {
        self.0.len().try_into().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_multiple(&self) -> bool {
        self.len() > 1
    }
}

impl<'de> Deserialize<'de> for FormAnswerItemFileSharings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormAnswerItemFileSharings::from_sharing_answers(Vec::<FileSharingAnswer>::deserialize(
            deserializer,
        )?)
        .map_err(de::Error::custom)
    }
}

// TODO: Keep consistency between shared file's type and `type_`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSharingAnswer {
    pub sharing_id: FileSharingId,
    pub type_: FileType,
}
