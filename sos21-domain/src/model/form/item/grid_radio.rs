use std::collections::HashSet;

use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form_answer::item::FormAnswerItemGridRows;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

pub mod column;
pub mod row;

pub use column::{GridRadioColumn, GridRadioColumnId, GridRadioColumnLabel};
pub use row::{GridRadioRow, GridRadioRowId, GridRadioRowLabel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridRadioFormItemRequired {
    All,
    None,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct GridRadioFormItemRows(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioRow>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromRowsErrorKind {
    Empty,
    TooLong,
    DuplicatedRowId { id: GridRadioRowId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item grid radio button row list")]
pub struct FromRowsError {
    kind: FromRowsErrorKind,
}

impl FromRowsError {
    pub fn kind(&self) -> FromRowsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::BoundedLengthError<typenum::U1, typenum::U32>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromRowsErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromRowsErrorKind::Empty,
        };
        FromRowsError { kind }
    }
}

#[allow(clippy::len_without_is_empty)]
impl GridRadioFormItemRows {
    pub fn from_rows<I>(rows: I) -> Result<Self, FromRowsError>
    where
        I: IntoIterator<Item = GridRadioRow>,
    {
        let rows = rows.into_iter();
        let capacity = rows.size_hint().0;

        let mut known_row_ids = HashSet::with_capacity(capacity);
        let mut result = Vec::with_capacity(capacity);
        for row in rows {
            if !known_row_ids.insert(row.id) {
                return Err(FromRowsError {
                    kind: FromRowsErrorKind::DuplicatedRowId { id: row.id },
                });
            }

            result.push(row);
        }

        let rows = LengthBoundedVec::new(result).map_err(FromRowsError::from_length_error)?;
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

impl<'de> Deserialize<'de> for GridRadioFormItemRows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        GridRadioFormItemRows::from_rows(Vec::<GridRadioRow>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct GridRadioFormItemColumns(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioColumn>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromColumnsErrorKind {
    Empty,
    TooLong,
    DuplicatedColumnId { id: GridRadioColumnId },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item grid radio button column list")]
pub struct FromColumnsError {
    kind: FromColumnsErrorKind,
}

impl FromColumnsError {
    pub fn kind(&self) -> FromColumnsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::BoundedLengthError<typenum::U1, typenum::U32>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromColumnsErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromColumnsErrorKind::Empty,
        };
        FromColumnsError { kind }
    }
}

#[allow(clippy::len_without_is_empty)]
impl GridRadioFormItemColumns {
    pub fn from_columns<I>(columns: I) -> Result<Self, FromColumnsError>
    where
        I: IntoIterator<Item = GridRadioColumn>,
    {
        let columns = columns.into_iter();
        let capacity = columns.size_hint().0;

        let mut known_column_ids = HashSet::with_capacity(capacity);
        let mut result = Vec::with_capacity(capacity);
        for column in columns {
            if !known_column_ids.insert(column.id) {
                return Err(FromColumnsError {
                    kind: FromColumnsErrorKind::DuplicatedColumnId { id: column.id },
                });
            }

            result.push(column);
        }

        let columns = LengthBoundedVec::new(result).map_err(FromColumnsError::from_length_error)?;
        Ok(GridRadioFormItemColumns(columns))
    }

    /// it always stands that `columns.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }

    pub fn columns(&self) -> impl Iterator<Item = &'_ GridRadioColumn> {
        self.0.iter()
    }

    pub fn into_columns(self) -> impl Iterator<Item = GridRadioColumn> {
        self.0.into_inner().into_iter()
    }
}

impl<'de> Deserialize<'de> for GridRadioFormItemColumns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        GridRadioFormItemColumns::from_columns(Vec::<GridRadioColumn>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioFormItemContent {
    pub rows: GridRadioFormItemRows,
    pub columns: GridRadioFormItemColumns,
    pub exclusive_column: bool,
    pub required: GridRadioFormItemRequired,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct GridRadioFormItem(GridRadioFormItemContent);

#[derive(Debug, Error, Clone)]
#[error("Columns must be more than rows when exclusive_column = true and required = All")]
pub struct TooFewColumnsError {
    _priv: (),
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
    pub fn from_content(content: GridRadioFormItemContent) -> Result<Self, TooFewColumnsError> {
        if content.exclusive_column
            && content.required == GridRadioFormItemRequired::All
            && content.rows.len() > content.columns.len()
        {
            return Err(TooFewColumnsError { _priv: () });
        }

        Ok(GridRadioFormItem(content))
    }

    pub fn into_content(self) -> GridRadioFormItemContent {
        self.0
    }

    pub fn check_answer(&self, answer: &FormAnswerItemGridRows) -> Result<(), CheckAnswerError> {
        let mut checked_columns = HashSet::new();

        if self.0.rows.len() != answer.len() {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::MismatchedRowsLength,
            });
        }

        for (row, row_answer) in self.rows().zip(answer.row_answers()) {
            if row.id != row_answer.row_id {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::MismatchedGridRadioRowId {
                        expected: row.id,
                        got: row_answer.row_id,
                    },
                });
            }

            match self.0.required {
                GridRadioFormItemRequired::All if row_answer.value.is_none() => {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAnsweredRows,
                    });
                }
                _ => {}
            }

            if let Some(column_id) = row_answer.value {
                if !self
                    .columns()
                    .any(|column| column.id == column_id)
                {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::UnknownGridRadioColumnId { id: column_id },
                    });
                }

                if !checked_columns.insert(column_id) && self.exclusive_column() {
                    return Err(CheckAnswerError {
                        kind: CheckAnswerErrorKind::NotAllowedDuplicatedColumn { id: column_id },
                    });
                }
            }
        }

        Ok(())
    }

    pub fn exclusive_column(&self) -> bool {
        self.0.exclusive_column
    }

    pub fn columns(&self) -> impl Iterator<Item = &'_ GridRadioColumn> {
        self.0.columns.columns()
    }

    pub fn rows(&self) -> impl Iterator<Item = &'_ GridRadioRow> {
        self.0.rows.rows()
    }
}

impl<'de> Deserialize<'de> for GridRadioFormItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        GridRadioFormItem::from_content(GridRadioFormItemContent::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckAnswerErrorKind, GridRadioFormItem, GridRadioFormItemColumns,
        GridRadioFormItemContent, GridRadioFormItemRequired, GridRadioFormItemRows,
        TooFewColumnsError,
    };
    use crate::test::model as test_model;

    #[test]
    fn test_pass() {
        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![
                test_model::new_form_grid_radio_row(),
                test_model::new_form_grid_radio_row(),
                test_model::new_form_grid_radio_row(),
            ])
            .unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![
                test_model::new_form_grid_radio_column(),
                test_model::new_form_grid_radio_column(),
            ])
            .unwrap(),
            exclusive_column: false,
            required: GridRadioFormItemRequired::All,
        })
        .unwrap();

        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![
                test_model::new_form_grid_radio_row(),
                test_model::new_form_grid_radio_row(),
                test_model::new_form_grid_radio_row(),
            ])
            .unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![
                test_model::new_form_grid_radio_column(),
                test_model::new_form_grid_radio_column(),
            ])
            .unwrap(),
            exclusive_column: true,
            required: GridRadioFormItemRequired::None,
        })
        .unwrap();

        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![
                test_model::new_form_grid_radio_row(),
                test_model::new_form_grid_radio_row(),
            ])
            .unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![
                test_model::new_form_grid_radio_column(),
                test_model::new_form_grid_radio_column(),
            ])
            .unwrap(),
            exclusive_column: true,
            required: GridRadioFormItemRequired::All,
        })
        .unwrap();
    }

    #[test]
    fn test_too_few_columns_when_exclusive_and_required() {
        assert!(matches!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![
                    test_model::new_form_grid_radio_row(),
                    test_model::new_form_grid_radio_row(),
                    test_model::new_form_grid_radio_row(),
                ])
                .unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![
                    test_model::new_form_grid_radio_column(),
                    test_model::new_form_grid_radio_column(),
                ])
                .unwrap(),
                exclusive_column: true,
                required: GridRadioFormItemRequired::All,
            }),
            Err(TooFewColumnsError { .. }),
        ));
    }

    #[test]
    fn test_answer_pass() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![column1.clone(), column2.clone()])
                .unwrap(),
            exclusive_column: true,
            required: GridRadioFormItemRequired::All,
        })
        .unwrap()
        .check_answer(
            &FormAnswerItemGridRows::from_row_answers(vec![
                GridRadioRowAnswer {
                    row_id: row1.id,
                    value: Some(column2.id),
                },
                GridRadioRowAnswer {
                    row_id: row2.id,
                    value: Some(column1.id),
                },
            ])
            .unwrap(),
        )
        .unwrap();

        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![column1.clone(), column2.clone()])
                .unwrap(),
            exclusive_column: false,
            required: GridRadioFormItemRequired::All,
        })
        .unwrap()
        .check_answer(
            &FormAnswerItemGridRows::from_row_answers(vec![
                GridRadioRowAnswer {
                    row_id: row1.id,
                    value: Some(column2.id),
                },
                GridRadioRowAnswer {
                    row_id: row2.id,
                    value: Some(column2.id),
                },
            ])
            .unwrap(),
        )
        .unwrap();

        GridRadioFormItem::from_content(GridRadioFormItemContent {
            rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
            columns: GridRadioFormItemColumns::from_columns(vec![column1.clone(), column2.clone()])
                .unwrap(),
            exclusive_column: true,
            required: GridRadioFormItemRequired::None,
        })
        .unwrap()
        .check_answer(
            &FormAnswerItemGridRows::from_row_answers(vec![
                GridRadioRowAnswer {
                    row_id: row1.id,
                    value: Some(column2.id),
                },
                GridRadioRowAnswer {
                    row_id: row2.id,
                    value: None,
                },
            ])
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_answer_mismatched_length() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        assert_eq!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![
                    column1.clone(),
                    column2.clone(),
                ])
                .unwrap(),
                exclusive_column: true,
                required: GridRadioFormItemRequired::All,
            })
            .unwrap()
            .check_answer(
                &FormAnswerItemGridRows::from_row_answers(vec![GridRadioRowAnswer {
                    row_id: row1.id,
                    value: Some(column2.id),
                }])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::MismatchedRowsLength,
        );
    }

    #[test]
    fn test_answer_mismatched_row_id() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let row3 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        assert!(matches!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![
                    column1.clone(),
                    column2.clone(),
                ])
                .unwrap(),
                exclusive_column: true,
                required: GridRadioFormItemRequired::All,
            })
            .unwrap()
            .check_answer(
                &FormAnswerItemGridRows::from_row_answers(vec![
                    GridRadioRowAnswer {
                        row_id: row3.id,
                        value: Some(column2.id),
                    },
                    GridRadioRowAnswer {
                        row_id: row1.id,
                        value: Some(column1.id),
                    },
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::MismatchedGridRadioRowId { .. },
        ));
    }

    #[test]
    fn test_answer_not_answered_rows() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        assert_eq!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![
                    column1.clone(),
                    column2.clone(),
                ])
                .unwrap(),
                exclusive_column: true,
                required: GridRadioFormItemRequired::All,
            })
            .unwrap()
            .check_answer(
                &FormAnswerItemGridRows::from_row_answers(vec![
                    GridRadioRowAnswer {
                        row_id: row1.id,
                        value: Some(column2.id),
                    },
                    GridRadioRowAnswer {
                        row_id: row2.id,
                        value: None,
                    }
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAnsweredRows,
        );
    }

    #[test]
    fn test_answer_unknown_column_id() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        assert_eq!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![column1.clone()]).unwrap(),
                exclusive_column: false,
                required: GridRadioFormItemRequired::None,
            })
            .unwrap()
            .check_answer(
                &FormAnswerItemGridRows::from_row_answers(vec![
                    GridRadioRowAnswer {
                        row_id: row1.id,
                        value: Some(column2.id),
                    },
                    GridRadioRowAnswer {
                        row_id: row2.id,
                        value: None,
                    }
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::UnknownGridRadioColumnId { id: column2.id },
        );
    }

    #[test]
    fn test_answer_exclusive() {
        use crate::model::form_answer::item::{FormAnswerItemGridRows, GridRadioRowAnswer};

        let row1 = test_model::new_form_grid_radio_row();
        let row2 = test_model::new_form_grid_radio_row();
        let column1 = test_model::new_form_grid_radio_column();
        let column2 = test_model::new_form_grid_radio_column();

        assert_eq!(
            GridRadioFormItem::from_content(GridRadioFormItemContent {
                rows: GridRadioFormItemRows::from_rows(vec![row1.clone(), row2.clone()]).unwrap(),
                columns: GridRadioFormItemColumns::from_columns(vec![
                    column1.clone(),
                    column2.clone(),
                ])
                .unwrap(),
                exclusive_column: true,
                required: GridRadioFormItemRequired::All,
            })
            .unwrap()
            .check_answer(
                &FormAnswerItemGridRows::from_row_answers(vec![
                    GridRadioRowAnswer {
                        row_id: row1.id,
                        value: Some(column2.id),
                    },
                    GridRadioRowAnswer {
                        row_id: row2.id,
                        value: Some(column2.id),
                    }
                ])
                .unwrap(),
            )
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedDuplicatedColumn { id: column2.id },
        );
    }
}
