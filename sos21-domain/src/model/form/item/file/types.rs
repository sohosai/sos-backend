use std::collections::HashSet;

use crate::model::collection::{self, LengthBoundedSet};
use crate::model::file::FileType;

use serde::{
    de::{self, Deserialize, Deserializer},
    Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct FileFormItemTypes(LengthBoundedSet<typenum::U1, typenum::U8, FileType>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromTypesErrorKind {
    Duplicated,
    TooLong,
    Empty,
}

#[derive(Debug, Error, Clone)]
#[error("invalid file form item types")]
pub struct FromTypesError {
    kind: FromTypesErrorKind,
}

impl FromTypesError {
    pub fn kind(&self) -> FromTypesErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::BoundedLengthError<typenum::U1, typenum::U8>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromTypesErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromTypesErrorKind::Empty,
        };
        FromTypesError { kind }
    }
}

impl FileFormItemTypes {
    pub fn from_types<I>(types: I) -> Result<Self, FromTypesError>
    where
        I: IntoIterator<Item = FileType>,
    {
        let mut result = HashSet::new();
        for type_ in types {
            if !result.insert(type_) {
                return Err(FromTypesError {
                    kind: FromTypesErrorKind::Duplicated,
                });
            }
        }
        let result = LengthBoundedSet::new(result).map_err(FromTypesError::from_length_error)?;
        Ok(FileFormItemTypes(result))
    }

    pub fn contains(&self, type_: &FileType) -> bool {
        self.0.contains(type_)
    }

    pub fn types(&self) -> impl Iterator<Item = &'_ FileType> {
        self.0.iter()
    }

    pub fn first(&self) -> &'_ FileType {
        // TODO: statically assert totality
        self.0.iter().next().unwrap()
    }

    pub fn into_types(self) -> impl Iterator<Item = FileType> {
        self.0.into_inner().into_iter()
    }
}

impl<'de> Deserialize<'de> for FileFormItemTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FileFormItemTypes::from_types(Vec::<FileType>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}
