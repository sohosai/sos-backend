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

    /// it always stands that `xs.buttons().next().is_some()`
    pub fn buttons(&self) -> impl Iterator<Item = &'_ Radio> {
        debug_assert!(self.0.iter().next().is_some());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    NotAnswered,
    UnknownRadioId { id: RadioId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer radio item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl RadioFormItem {
    pub fn check_answer(&self, answer: Option<RadioId>) -> Result<(), CheckAnswerError> {
        let answer = match (self.is_required, answer) {
            (true, None) => {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::NotAnswered,
                })
            }
            (false, None) => return Ok(()),
            (_, Some(answer)) => answer,
        };

        if self
            .buttons
            .buttons()
            .find(|button| button.id == answer)
            .is_none()
        {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::UnknownRadioId { id: answer },
            });
        }

        Ok(())
    }
}
