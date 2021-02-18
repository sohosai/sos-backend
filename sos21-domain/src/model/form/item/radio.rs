use crate::model::collection::{self, LengthBoundedVec};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(clippy::module_inception)]
pub mod radio;
pub use radio::{Radio, RadioId, RadioLabel};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RadioFormItemButtons(LengthBoundedVec<typenum::U1, typenum::U32, Radio>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    Empty,
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item radio button list")]
pub struct LengthError {
    kind: LengthErrorKind,
}

impl LengthError {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => LengthErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => LengthErrorKind::Empty,
        };
        LengthError { kind }
    }
}

impl RadioFormItemButtons {
    pub fn from_buttons<I>(buttons: I) -> Result<Self, LengthError>
    where
        I: IntoIterator<Item = Radio>,
    {
        let buttons = LengthBoundedVec::new(buttons.into_iter().collect())
            .map_err(LengthError::from_length_error)?;
        Ok(RadioFormItemButtons(buttons))
    }

    pub fn buttons(&self) -> impl Iterator<Item = &'_ Radio> {
        self.0.iter()
    }

    pub fn into_buttons(self) -> impl Iterator<Item = Radio> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioFormItem {
    pub buttons: RadioFormItemButtons,
    pub is_required: bool,
}
