use std::collections::HashSet;

use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form::item::grid_radio::{GridRadioColumnId, GridRadioRowId};

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct FormAnswerItemGridRows(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioRowAnswer>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromRowAnswersErrorKind {
    Empty,
    TooLong,
    DuplicatedRowId { id: GridRadioRowId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer item grid row answers")]
pub struct FromRowAnswersError {
    kind: FromRowAnswersErrorKind,
}

impl FromRowAnswersError {
    pub fn kind(&self) -> FromRowAnswersErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::BoundedLengthError<typenum::U1, typenum::U32>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromRowAnswersErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromRowAnswersErrorKind::Empty,
        };
        FromRowAnswersError { kind }
    }
}

#[allow(clippy::len_without_is_empty)]
impl FormAnswerItemGridRows {
    pub fn from_row_answers<I>(answers: I) -> Result<Self, FromRowAnswersError>
    where
        I: IntoIterator<Item = GridRadioRowAnswer>,
    {
        let answers = answers.into_iter();
        let capacity = answers.size_hint().0;

        let mut known_row_ids = HashSet::with_capacity(capacity);
        let mut result = Vec::with_capacity(capacity);
        for answer in answers {
            if !known_row_ids.insert(answer.row_id) {
                return Err(FromRowAnswersError {
                    kind: FromRowAnswersErrorKind::DuplicatedRowId { id: answer.row_id },
                });
            }

            result.push(answer);
        }

        let answers =
            LengthBoundedVec::new(result).map_err(FromRowAnswersError::from_length_error)?;
        Ok(FormAnswerItemGridRows(answers))
    }

    /// it always stands that `answers.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }

    pub fn row_answers(&self) -> impl Iterator<Item = &'_ GridRadioRowAnswer> {
        self.0.iter()
    }

    pub fn into_row_answers(self) -> impl Iterator<Item = GridRadioRowAnswer> {
        self.0.into_inner().into_iter()
    }
}

impl<'de> Deserialize<'de> for FormAnswerItemGridRows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormAnswerItemGridRows::from_row_answers(Vec::<GridRadioRowAnswer>::deserialize(
            deserializer,
        )?)
        .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridRadioRowAnswer {
    pub row_id: GridRadioRowId,
    pub value: Option<GridRadioColumnId>,
}
