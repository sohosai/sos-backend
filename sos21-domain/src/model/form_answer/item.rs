use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form::item::radio::RadioId;
use crate::model::form::item::FormItemId;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod checks;
pub mod file_sharings;
pub mod grid_rows;
pub mod text;
pub use checks::FormAnswerItemChecks;
pub use file_sharings::{FileSharingAnswer, FormAnswerItemFileSharings};
pub use grid_rows::{FormAnswerItemGridRows, GridRadioRowAnswer};
pub use text::FormAnswerItemText;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItems(LengthBoundedVec<typenum::U1, typenum::U64, FormAnswerItem>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    Empty,
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer item list")]
pub struct LengthError {
    kind: LengthErrorKind,
}

impl LengthError {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::BoundedLengthError<typenum::U1, typenum::U64>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => LengthErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => LengthErrorKind::Empty,
        };
        LengthError { kind }
    }
}

#[allow(clippy::len_without_is_empty)]
impl FormAnswerItems {
    pub fn from_items<I>(items: I) -> Result<Self, LengthError>
    where
        I: IntoIterator<Item = FormAnswerItem>,
    {
        let items = items.into_iter().collect();
        let items = LengthBoundedVec::new(items).map_err(LengthError::from_length_error)?;
        Ok(FormAnswerItems(items))
    }

    /// it always stands that `items.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }

    pub fn items(&self) -> impl Iterator<Item = &FormAnswerItem> {
        self.0.iter()
    }

    pub fn into_items(self) -> impl Iterator<Item = FormAnswerItem> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormAnswerItem {
    pub item_id: FormItemId,
    pub body: Option<FormAnswerItemBody>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormAnswerItemBody {
    Text(Option<FormAnswerItemText>),
    Integer(Option<u64>),
    Checkbox(FormAnswerItemChecks),
    Radio(Option<RadioId>),
    GridRadio(FormAnswerItemGridRows),
    File(FormAnswerItemFileSharings),
}
