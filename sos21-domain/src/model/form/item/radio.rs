use std::collections::HashSet;

use crate::model::collection::{self, LengthBoundedVec};

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[allow(clippy::module_inception)]
pub mod radio;
pub use radio::{Radio, RadioId, RadioLabel};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct RadioFormItemButtons(LengthBoundedVec<typenum::U1, typenum::U32, Radio>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromButtonsErrorKind {
    Empty,
    TooLong,
    DuplicatedRadioId { id: RadioId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item radio button list")]
pub struct FromButtonsError {
    kind: FromButtonsErrorKind,
}

impl FromButtonsError {
    pub fn kind(&self) -> FromButtonsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromButtonsErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromButtonsErrorKind::Empty,
        };
        FromButtonsError { kind }
    }
}

impl RadioFormItemButtons {
    pub fn from_buttons<I>(buttons: I) -> Result<Self, FromButtonsError>
    where
        I: IntoIterator<Item = Radio>,
    {
        let buttons = buttons.into_iter();
        let capacity = buttons.size_hint().0;

        let mut known_button_ids = HashSet::with_capacity(capacity);
        let mut result = Vec::with_capacity(capacity);

        for button in buttons {
            if !known_button_ids.insert(button.id) {
                return Err(FromButtonsError {
                    kind: FromButtonsErrorKind::DuplicatedRadioId { id: button.id },
                });
            }

            result.push(button);
        }

        let buttons = LengthBoundedVec::new(result).map_err(FromButtonsError::from_length_error)?;
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

impl<'de> Deserialize<'de> for RadioFormItemButtons {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RadioFormItemButtons::from_buttons(Vec::<Radio>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
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
