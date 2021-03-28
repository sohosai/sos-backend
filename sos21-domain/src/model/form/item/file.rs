use crate::model::form_answer::item::FormAnswerItemFileSharings;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod types;
pub use types::FileFormItemTypes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFormItem {
    pub types: Option<FileFormItemTypes>,
    pub accept_multiple_files: bool,
    pub is_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    NotAnswered,
    NotAllowedMultipleFiles,
    NotAllowedFileType,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer file item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl FileFormItem {
    pub fn check_answer(
        &self,
        answer: &FormAnswerItemFileSharings,
    ) -> Result<(), CheckAnswerError> {
        if answer.is_multiple() && !self.accept_multiple_files {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::NotAllowedMultipleFiles,
            });
        }

        if answer.is_empty() && self.is_required {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::NotAnswered,
            });
        }

        if let Some(types) = &self.types {
            for sharing_answer in answer.sharing_answers() {
                if !types.contains(&sharing_answer.type_) {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAllowedFileType,
                    });
                }
            }
        }

        Ok(())
    }
}
