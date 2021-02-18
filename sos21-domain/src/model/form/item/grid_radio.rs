use crate::model::collection::{self, LengthBoundedVec};

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

impl GridRadioFormItemRows {
    pub fn from_rows<I>(rows: I) -> Result<Self, RowsLengthError>
    where
        I: IntoIterator<Item = GridRadioRow>,
    {
        let rows = rows.into_iter().collect();
        let rows = LengthBoundedVec::new(rows).map_err(RowsLengthError::from_length_error)?;
        Ok(GridRadioFormItemRows(rows))
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
