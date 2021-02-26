use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form_answer::item::FormAnswerItemChecks;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[allow(clippy::module_inception)]
pub mod checkbox;
pub mod limit;

pub use checkbox::{Checkbox, CheckboxId, CheckboxLabel};
pub use limit::CheckboxFormItemLimit;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckboxFormItemBoxes(LengthBoundedVec<typenum::U1, typenum::U32, Checkbox>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    Empty,
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid checkbox form item box list")]
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

#[allow(clippy::len_without_is_empty)]
impl CheckboxFormItemBoxes {
    pub fn from_boxes<I>(boxes: I) -> Result<Self, LengthError>
    where
        I: IntoIterator<Item = Checkbox>,
    {
        let boxes = LengthBoundedVec::new(boxes.into_iter().collect())
            .map_err(LengthError::from_length_error)?;
        Ok(CheckboxFormItemBoxes(boxes))
    }

    pub fn boxes(&self) -> impl Iterator<Item = &'_ Checkbox> {
        self.0.iter()
    }

    pub fn into_boxes(self) -> impl Iterator<Item = Checkbox> {
        self.0.into_inner().into_iter()
    }

    /// it always stands that `boxes.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxFormItemContent {
    pub boxes: CheckboxFormItemBoxes,
    pub min_checks: Option<CheckboxFormItemLimit>,
    pub max_checks: Option<CheckboxFormItemLimit>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct CheckboxFormItem(CheckboxFormItemContent);

#[derive(Debug, Error, Clone)]
#[error("invalid checkbox form item limit length")]
pub struct LimitError {
    _priv: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    TooManyChecks,
    TooLittleChecks,
    UnknownCheckboxId { id: CheckboxId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer checkbox item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl CheckboxFormItem {
    pub fn from_content(content: CheckboxFormItemContent) -> Result<Self, LimitError> {
        if let Some(min_checks) = &content.min_checks {
            if min_checks.to_u64() > content.boxes.len() as u64 {
                return Err(LimitError { _priv: () });
            }
        }

        if let Some(max_checks) = &content.max_checks {
            if max_checks.to_u64() < content.boxes.len() as u64 {
                return Err(LimitError { _priv: () });
            }
        }

        Ok(CheckboxFormItem(content))
    }

    pub fn min_checks(&self) -> Option<u64> {
        self.0.min_checks.map(CheckboxFormItemLimit::to_u64)
    }

    pub fn max_checks(&self) -> Option<u64> {
        self.0.max_checks.map(CheckboxFormItemLimit::to_u64)
    }

    pub fn into_content(self) -> CheckboxFormItemContent {
        self.0
    }

    pub fn boxes(&self) -> impl Iterator<Item = &'_ Checkbox> {
        self.0.boxes.boxes()
    }

    pub fn check_answer(&self, answer: &FormAnswerItemChecks) -> Result<(), CheckAnswerError> {
        if let Some(max_checks) = self.0.max_checks {
            if max_checks.to_u64() < answer.count_checks() as u64 {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooManyChecks,
                });
            }
        }

        if let Some(min_checks) = self.0.min_checks {
            if min_checks.to_u64() < answer.count_checks() as u64 {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooLittleChecks,
                });
            }
        }

        for check_id in answer.checked_ids() {
            if self
                .boxes()
                .find(|checkbox| checkbox.id == check_id)
                .is_none()
            {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::UnknownCheckboxId { id: check_id },
                });
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for CheckboxFormItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        CheckboxFormItem::from_content(CheckboxFormItemContent::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}
