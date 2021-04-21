use crate::model::form::item::{
    CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId,
};
use sos21_domain::model::form;

#[derive(Debug, Clone)]
pub enum CheckAnswerError {
    MismatchedItemsLength,
    MismatchedItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    InvalidAnswerItem {
        item_id: FormItemId,
        item_error: CheckAnswerItemError,
    },
}

pub fn to_check_answer_error(err: form::item::CheckAnswerError) -> CheckAnswerError {
    match err.kind() {
        form::item::CheckAnswerErrorKind::MismatchedItemsLength => {
            CheckAnswerError::MismatchedItemsLength
        }
        form::item::CheckAnswerErrorKind::MismatchedItemId { expected, got } => {
            CheckAnswerError::MismatchedItemId {
                expected: FormItemId::from_entity(expected),
                got: FormItemId::from_entity(got),
            }
        }
        form::item::CheckAnswerErrorKind::Item(item_id, err) => {
            CheckAnswerError::InvalidAnswerItem {
                item_id: FormItemId::from_entity(item_id),
                item_error: to_check_answer_item_error(err),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheckAnswerItemError {
    NotAnsweredWithoutCondition,
    NotAnsweredWithCondition,
    UnexpectedAnswer,
    MismatchedItemType,
    NotAnsweredText,
    TooLongText,
    TooShortText,
    NotAllowedMultipleLineText,
    NotAnsweredInteger,
    TooBigInteger,
    TooSmallInteger,
    TooManyChecks,
    TooFewChecks,
    NotAnsweredFile,
    NotAllowedMultipleFiles,
    NotAllowedFileType,
    UnknownCheckboxId {
        id: CheckboxId,
    },
    NotAnsweredRadio,
    UnknownRadioId {
        id: RadioId,
    },
    NotAnsweredGridRadioRows,
    MismatchedGridRadioRowsLength,
    MismatchedGridRadioRowId {
        expected: GridRadioRowId,
        got: GridRadioRowId,
    },
    UnknownGridRadioColumnId {
        id: GridRadioColumnId,
    },
    NotAllowedDuplicatedGridRadioColumn {
        id: GridRadioColumnId,
    },
}

pub fn to_check_answer_item_error(
    kind: form::item::CheckAnswerItemErrorKind,
) -> CheckAnswerItemError {
    match kind {
        form::item::CheckAnswerItemErrorKind::NotAnsweredWithoutCondition => {
            CheckAnswerItemError::NotAnsweredWithoutCondition
        }
        form::item::CheckAnswerItemErrorKind::NotAnsweredWithCondition => {
            CheckAnswerItemError::NotAnsweredWithCondition
        }
        form::item::CheckAnswerItemErrorKind::UnexpectedAnswer => {
            CheckAnswerItemError::UnexpectedAnswer
        }
        form::item::CheckAnswerItemErrorKind::MismatchedItemType => {
            CheckAnswerItemError::MismatchedItemType
        }
        form::item::CheckAnswerItemErrorKind::NotAnsweredText => {
            CheckAnswerItemError::NotAnsweredText
        }
        form::item::CheckAnswerItemErrorKind::TooLongText => CheckAnswerItemError::TooLongText,
        form::item::CheckAnswerItemErrorKind::TooShortText => CheckAnswerItemError::TooShortText,
        form::item::CheckAnswerItemErrorKind::NotAllowedMultipleLineText => {
            CheckAnswerItemError::NotAllowedMultipleLineText
        }
        form::item::CheckAnswerItemErrorKind::NotAnsweredInteger => {
            CheckAnswerItemError::NotAnsweredInteger
        }
        form::item::CheckAnswerItemErrorKind::TooBigInteger => CheckAnswerItemError::TooBigInteger,
        form::item::CheckAnswerItemErrorKind::TooSmallInteger => {
            CheckAnswerItemError::TooSmallInteger
        }
        form::item::CheckAnswerItemErrorKind::TooManyChecks => CheckAnswerItemError::TooManyChecks,
        form::item::CheckAnswerItemErrorKind::TooFewChecks => CheckAnswerItemError::TooFewChecks,
        form::item::CheckAnswerItemErrorKind::NotAnsweredRadio => {
            CheckAnswerItemError::NotAnsweredRadio
        }
        form::item::CheckAnswerItemErrorKind::NotAnsweredGridRadioRows => {
            CheckAnswerItemError::NotAnsweredGridRadioRows
        }
        form::item::CheckAnswerItemErrorKind::NotAnsweredFile => {
            CheckAnswerItemError::NotAnsweredFile
        }
        form::item::CheckAnswerItemErrorKind::NotAllowedMultipleFiles => {
            CheckAnswerItemError::NotAllowedMultipleFiles
        }
        form::item::CheckAnswerItemErrorKind::NotAllowedFileType => {
            CheckAnswerItemError::NotAllowedFileType
        }
        form::item::CheckAnswerItemErrorKind::UnknownCheckboxId { id } => {
            CheckAnswerItemError::UnknownCheckboxId {
                id: CheckboxId::from_entity(id),
            }
        }
        form::item::CheckAnswerItemErrorKind::UnknownRadioId { id } => {
            CheckAnswerItemError::UnknownRadioId {
                id: RadioId::from_entity(id),
            }
        }
        form::item::CheckAnswerItemErrorKind::MismatchedGridRadioRowsLength => {
            CheckAnswerItemError::MismatchedGridRadioRowsLength
        }
        form::item::CheckAnswerItemErrorKind::MismatchedGridRadioRowId { expected, got } => {
            CheckAnswerItemError::MismatchedGridRadioRowId {
                expected: GridRadioRowId::from_entity(expected),
                got: GridRadioRowId::from_entity(got),
            }
        }
        form::item::CheckAnswerItemErrorKind::UnknownGridRadioColumnId { id } => {
            CheckAnswerItemError::UnknownGridRadioColumnId {
                id: GridRadioColumnId::from_entity(id),
            }
        }
        form::item::CheckAnswerItemErrorKind::NotAllowedDuplicatedGridRadioColumn { id } => {
            CheckAnswerItemError::NotAllowedDuplicatedGridRadioColumn {
                id: GridRadioColumnId::from_entity(id),
            }
        }
    }
}
