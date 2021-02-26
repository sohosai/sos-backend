use std::collections::HashSet;

use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form_answer::item::FormAnswerItemGridRows;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod column;
pub mod row;

pub use column::{GridRadioColumn, GridRadioColumnId, GridRadioColumnLabel};
pub use row::{GridRadioRow, GridRadioRowId, GridRadioRowLabel};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridRadioFormItemRequired {
    All,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GridRadioFormItemRows(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioRow>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    Empty,
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item grid radio button row list")]
pub struct RowsLengthError {
    kind: LengthErrorKind,
}

impl RowsLengthError {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => LengthErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => LengthErrorKind::Empty,
        };
        RowsLengthError { kind }
    }
}

#[allow(clippy::len_without_is_empty)]
impl GridRadioFormItemRows {
    pub fn from_rows<I>(rows: I) -> Result<Self, RowsLengthError>
    where
        I: IntoIterator<Item = GridRadioRow>,
    {
        let rows = rows.into_iter().collect();
        let rows = LengthBoundedVec::new(rows).map_err(RowsLengthError::from_length_error)?;
        Ok(GridRadioFormItemRows(rows))
    }

    /// it always stands that `rows.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }

    pub fn rows(&self) -> impl Iterator<Item = &'_ GridRadioRow> {
        self.0.iter()
    }

    pub fn into_rows(self) -> impl Iterator<Item = GridRadioRow> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GridRadioFormItemColumns(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioColumn>);

#[derive(Debug, Error, Clone)]
#[error("invalid form item grid radio button column list")]
pub struct ColumnsLengthError {
    kind: LengthErrorKind,
}

impl ColumnsLengthError {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => LengthErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => LengthErrorKind::Empty,
        };
        ColumnsLengthError { kind }
    }
}

impl GridRadioFormItemColumns {
    pub fn from_columns<I>(columns: I) -> Result<Self, ColumnsLengthError>
    where
        I: IntoIterator<Item = GridRadioColumn>,
    {
        let columns = columns.into_iter().collect();
        let columns =
            LengthBoundedVec::new(columns).map_err(ColumnsLengthError::from_length_error)?;
        Ok(GridRadioFormItemColumns(columns))
    }

    pub fn columns(&self) -> impl Iterator<Item = &'_ GridRadioColumn> {
        self.0.iter()
    }

    pub fn into_columns(self) -> impl Iterator<Item = GridRadioColumn> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioFormItem {
    pub rows: GridRadioFormItemRows,
    pub columns: GridRadioFormItemColumns,
    pub exclusive_column: bool,
    pub required: GridRadioFormItemRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    MismatchedRowsLength,
    MismatchedGridRadioRowId {
        expected: GridRadioRowId,
        got: GridRadioRowId,
    },
    UnknownGridRadioColumnId {
        id: GridRadioColumnId,
    },
    NotAnsweredRows,
    NotAllowedDuplicatedColumn {
        id: GridRadioColumnId,
    },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer grid radio item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl GridRadioFormItem {
    pub fn check_answer(&self, answer: &FormAnswerItemGridRows) -> Result<(), CheckAnswerError> {
        let mut checked_columns = HashSet::new();

        if self.rows.len() != answer.len() {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::MismatchedRowsLength,
            });
        }

        for (row, row_answer) in self.rows.rows().zip(answer.row_answers()) {
            if row.id != row_answer.row_id {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::MismatchedGridRadioRowId {
                        expected: row.id,
                        got: row_answer.row_id,
                    },
                });
            }

            match self.required {
                GridRadioFormItemRequired::All if row_answer.value.is_none() => {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAnsweredRows,
                    });
                }
                _ => {}
            }

            if let Some(column_id) = row_answer.value {
                if self
                    .columns
                    .columns()
                    .find(|column| column.id == column_id)
                    .is_none()
                {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::UnknownGridRadioColumnId { id: column_id },
                    });
                }

                if !checked_columns.insert(column_id) && self.exclusive_column {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAllowedDuplicatedColumn { id: column_id },
                    });
                }
            }
        }

        Ok(())
    }

    pub fn exclusive_column(&self) -> bool {
        self.exclusive_column
    }

    pub fn columns(&self) -> impl Iterator<Item = &'_ GridRadioColumn> {
        self.columns.columns()
    }

    pub fn rows(&self) -> impl Iterator<Item = &'_ GridRadioRow> {
        self.rows.rows()
    }
}
