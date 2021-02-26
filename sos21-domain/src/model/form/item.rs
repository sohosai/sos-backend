use std::collections::{HashMap, HashSet};

use crate::model::collection::{self, LengthBoundedVec};
use crate::model::form_answer::item::{FormAnswerItem, FormAnswerItemBody, FormAnswerItems};

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;
use uuid::Uuid;

pub mod checkbox;
pub mod condition;
pub mod description;
pub mod grid_radio;
pub mod integer;
pub mod name;
pub mod radio;
pub mod text;
pub use checkbox::CheckboxFormItem;
pub use condition::{FormItemCondition, FormItemConditions};
pub use description::FormItemDescription;
pub use grid_radio::GridRadioFormItem;
pub use integer::IntegerFormItem;
pub use name::FormItemName;
pub use radio::RadioFormItem;
pub use text::TextFormItem;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct FormItems(LengthBoundedVec<typenum::U1, typenum::U64, FormItem>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromItemsErrorKind {
    Empty,
    TooLong,
    DuplicatedFormItemId(FormItemId),
    DuplicatedCheckboxId(checkbox::CheckboxId),
    DuplicatedRadioId(radio::RadioId),
    DuplicatedGridRadioRowId(grid_radio::GridRadioRowId),
    DuplicatedGridRadioColumnId(grid_radio::GridRadioColumnId),
    MismatchedConditionType {
        provenance: FormItemId,
        id: FormItemId,
    },
    UnknownFormItemIdInConditions {
        provenance: FormItemId,
        id: FormItemId,
    },
    UnknownCheckboxIdInConditions {
        provenance: FormItemId,
        id: checkbox::CheckboxId,
    },
    UnknownRadioIdInConditions {
        provenance: FormItemId,
        id: radio::RadioId,
    },
    UnknownGridRadioColumnIdInConditions {
        provenance: FormItemId,
        id: grid_radio::GridRadioColumnId,
    },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form item list")]
pub struct FromItemsError {
    kind: FromItemsErrorKind,
}

impl FromItemsError {
    pub fn kind(&self) -> FromItemsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromItemsErrorKind::TooLong,
            collection::LengthErrorKind::TooShort => FromItemsErrorKind::Empty,
        };
        FromItemsError { kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    MismatchedItemsLength,
    MismatchedItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    Item(FormItemId, CheckAnswerItemErrorKind),
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }

    fn from_item_error(item_id: FormItemId, err: CheckAnswerItemError) -> Self {
        CheckAnswerError {
            kind: CheckAnswerErrorKind::Item(item_id, err.kind()),
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl FormItems {
    pub fn from_items<I>(items: I) -> Result<Self, FromItemsError>
    where
        I: IntoIterator<Item = FormItem>,
    {
        let items = items.into_iter().collect();
        CheckFormItems::default().check_items(&items)?;
        let items = LengthBoundedVec::new(items).map_err(FromItemsError::from_length_error)?;
        Ok(FormItems(items))
    }

    /// it always stands that `items.len() > 0`.
    pub fn len(&self) -> usize {
        let len = self.0.len();
        debug_assert!(len > 0);
        len
    }

    pub fn items(&self) -> impl Iterator<Item = &FormItem> {
        self.0.iter()
    }

    pub fn into_items(self) -> impl Iterator<Item = FormItem> {
        self.0.into_inner().into_iter()
    }

    pub fn check_answer(
        &self,
        answer: &FormAnswerItems,
    ) -> Result<Result<(), CheckAnswerError>, anyhow::Error> {
        if self.len() != answer.len() {
            return Ok(Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::MismatchedItemsLength,
            }));
        }

        let mut known_answers = HashMap::new();
        for (item, answer_item) in self.items().zip(answer.items()) {
            if item.id != answer_item.item_id {
                return Ok(Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::MismatchedItemId {
                        expected: item.id,
                        got: answer_item.item_id,
                    },
                }));
            }

            if let Err(err) = item.check_answer(&known_answers, answer_item)? {
                return Ok(Err(CheckAnswerError::from_item_error(item.id, err)));
            }

            known_answers.insert(answer_item.item_id, answer_item.clone());
        }

        Ok(Ok(()))
    }
}

impl<'de> Deserialize<'de> for FormItems {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormItems::from_items(Vec::<FormItem>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerItemErrorKind {
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
    UnknownCheckboxId {
        id: checkbox::CheckboxId,
    },
    NotAnsweredRadio,
    UnknownRadioId {
        id: radio::RadioId,
    },
    NotAnsweredGridRadioRows,
    MismatchedGridRadioRowsLength,
    MismatchedGridRadioRowId {
        expected: grid_radio::GridRadioRowId,
        got: grid_radio::GridRadioRowId,
    },
    UnknownGridRadioColumnId {
        id: grid_radio::GridRadioColumnId,
    },
    NotAllowedDuplicatedGridRadioColumn {
        id: grid_radio::GridRadioColumnId,
    },
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer item")]
pub struct CheckAnswerItemError {
    kind: CheckAnswerItemErrorKind,
}

impl CheckAnswerItemError {
    pub fn kind(&self) -> CheckAnswerItemErrorKind {
        self.kind
    }

    pub fn from_text_item_error(err: text::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            text::CheckAnswerErrorKind::NotAnswered => CheckAnswerItemErrorKind::NotAnsweredText,
            text::CheckAnswerErrorKind::TooLong => CheckAnswerItemErrorKind::TooLongText,
            text::CheckAnswerErrorKind::TooShort => CheckAnswerItemErrorKind::TooShortText,
            text::CheckAnswerErrorKind::NotAllowedMultipleLine => {
                CheckAnswerItemErrorKind::NotAllowedMultipleLineText
            }
        };

        CheckAnswerItemError { kind }
    }

    pub fn from_integer_item_error(err: integer::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            integer::CheckAnswerErrorKind::NotAnswered => {
                CheckAnswerItemErrorKind::NotAnsweredInteger
            }
            integer::CheckAnswerErrorKind::TooBig => CheckAnswerItemErrorKind::TooBigInteger,
            integer::CheckAnswerErrorKind::TooSmall => CheckAnswerItemErrorKind::TooSmallInteger,
        };

        CheckAnswerItemError { kind }
    }

    pub fn from_checkbox_item_error(err: checkbox::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            checkbox::CheckAnswerErrorKind::TooManyChecks => {
                CheckAnswerItemErrorKind::TooManyChecks
            }
            checkbox::CheckAnswerErrorKind::TooFewChecks => CheckAnswerItemErrorKind::TooFewChecks,
            checkbox::CheckAnswerErrorKind::UnknownCheckboxId { id } => {
                CheckAnswerItemErrorKind::UnknownCheckboxId { id }
            }
        };

        CheckAnswerItemError { kind }
    }

    pub fn from_radio_item_error(err: radio::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            radio::CheckAnswerErrorKind::NotAnswered => CheckAnswerItemErrorKind::NotAnsweredRadio,
            radio::CheckAnswerErrorKind::UnknownRadioId { id } => {
                CheckAnswerItemErrorKind::UnknownRadioId { id }
            }
        };

        CheckAnswerItemError { kind }
    }

    pub fn from_grid_radio_item_error(err: grid_radio::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            grid_radio::CheckAnswerErrorKind::NotAnsweredRows => {
                CheckAnswerItemErrorKind::NotAnsweredGridRadioRows
            }
            grid_radio::CheckAnswerErrorKind::MismatchedRowsLength => {
                CheckAnswerItemErrorKind::MismatchedGridRadioRowsLength
            }
            grid_radio::CheckAnswerErrorKind::MismatchedGridRadioRowId { got, expected } => {
                CheckAnswerItemErrorKind::MismatchedGridRadioRowId { got, expected }
            }
            grid_radio::CheckAnswerErrorKind::UnknownGridRadioColumnId { id } => {
                CheckAnswerItemErrorKind::UnknownGridRadioColumnId { id }
            }
            grid_radio::CheckAnswerErrorKind::NotAllowedDuplicatedColumn { id } => {
                CheckAnswerItemErrorKind::NotAllowedDuplicatedGridRadioColumn { id }
            }
        };

        CheckAnswerItemError { kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormItemId(Uuid);

impl FormItemId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        FormItemId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormItem {
    pub id: FormItemId,
    pub name: FormItemName,
    pub description: FormItemDescription,
    pub conditions: Option<FormItemConditions>,
    pub body: FormItemBody,
}

impl FormItem {
    fn check_answer(
        &self,
        known_answers: &HashMap<FormItemId, FormAnswerItem>,
        answer: &FormAnswerItem,
    ) -> Result<Result<(), CheckAnswerItemError>, anyhow::Error> {
        let body = match (self.conditions.as_ref(), answer.body.as_ref()) {
            (None, None) => {
                return Ok(Err(CheckAnswerItemError {
                    kind: CheckAnswerItemErrorKind::NotAnsweredWithoutCondition,
                }))
            }
            (None, Some(body)) => body,
            (Some(conditions), body_opt) => {
                let is_match = conditions.is_matched_in(known_answers)?;
                match (is_match, body_opt) {
                    (true, Some(body)) => body,
                    (true, None) => {
                        return Ok(Err(CheckAnswerItemError {
                            kind: CheckAnswerItemErrorKind::NotAnsweredWithCondition,
                        }))
                    }
                    (false, Some(_)) => {
                        return Ok(Err(CheckAnswerItemError {
                            kind: CheckAnswerItemErrorKind::UnexpectedAnswer,
                        }))
                    }
                    (false, None) => return Ok(Ok(())),
                }
            }
        };

        Ok(self.body.check_answer(&body))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormItemBody {
    Text(TextFormItem),
    Integer(IntegerFormItem),
    Checkbox(CheckboxFormItem),
    Radio(RadioFormItem),
    GridRadio(GridRadioFormItem),
}

impl FormItemBody {
    pub fn check_answer(&self, answer: &FormAnswerItemBody) -> Result<(), CheckAnswerItemError> {
        match (self, answer) {
            (FormItemBody::Text(item), FormAnswerItemBody::Text(answer)) => item
                .check_answer(answer.as_ref())
                .map_err(CheckAnswerItemError::from_text_item_error),
            (FormItemBody::Integer(item), FormAnswerItemBody::Integer(answer)) => item
                .check_answer(*answer)
                .map_err(CheckAnswerItemError::from_integer_item_error),
            (FormItemBody::Checkbox(item), FormAnswerItemBody::Checkbox(answer)) => item
                .check_answer(answer)
                .map_err(CheckAnswerItemError::from_checkbox_item_error),
            (FormItemBody::Radio(item), FormAnswerItemBody::Radio(answer)) => item
                .check_answer(*answer)
                .map_err(CheckAnswerItemError::from_radio_item_error),
            (FormItemBody::GridRadio(item), FormAnswerItemBody::GridRadio(answer)) => item
                .check_answer(answer)
                .map_err(CheckAnswerItemError::from_grid_radio_item_error),
            (_, _) => Err(CheckAnswerItemError {
                kind: CheckAnswerItemErrorKind::MismatchedItemType,
            }),
        }
    }
}

#[derive(Default)]
struct CheckFormItems {
    items: HashMap<FormItemId, FormItem>,
    boxes: HashSet<checkbox::CheckboxId>,
    buttons: HashSet<radio::RadioId>,
    grid_rows: HashSet<grid_radio::GridRadioRowId>,
    grid_columns: HashSet<grid_radio::GridRadioColumnId>,
}

impl CheckFormItems {
    fn check_items<'a, I>(&mut self, items: I) -> Result<(), FromItemsError>
    where
        I: IntoIterator<Item = &'a FormItem>,
    {
        for item in items {
            self.check_item(item)?;
        }
        Ok(())
    }

    fn check_item(&mut self, item: &FormItem) -> Result<(), FromItemsError> {
        if self.items.insert(item.id, item.clone()).is_some() {
            return Err(FromItemsError {
                kind: FromItemsErrorKind::DuplicatedFormItemId(item.id),
            });
        }

        if let Some(conditions) = &item.conditions {
            self.check_conditions(item.id, conditions)?;
        }

        self.check_body(&item.body)
    }

    fn check_conditions(
        &mut self,
        item_id: FormItemId,
        conditions: &FormItemConditions,
    ) -> Result<(), FromItemsError> {
        for conj in conditions.conjunctions() {
            for condition in conj {
                self.check_condition(item_id, &condition)?;
            }
        }

        Ok(())
    }

    fn check_condition(
        &mut self,
        provenance: FormItemId,
        condition: &FormItemCondition,
    ) -> Result<(), FromItemsError> {
        match condition {
            FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected: _,
            } => self.check_checkbox_condition(provenance, *item_id, *checkbox_id),
            FormItemCondition::RadioSelected { item_id, radio_id } => {
                self.check_radio_condition(provenance, *item_id, *radio_id)
            }
            FormItemCondition::GridRadioSelected { item_id, column_id } => {
                self.check_grid_radio_condition(provenance, *item_id, *column_id)
            }
        }
    }

    fn check_checkbox_condition(
        &mut self,
        provenance: FormItemId,
        target_id: FormItemId,
        checkbox_id: checkbox::CheckboxId,
    ) -> Result<(), FromItemsError> {
        let item = match self.items.get(&target_id) {
            Some(item) => item,
            None => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::UnknownFormItemIdInConditions {
                        provenance,
                        id: target_id,
                    },
                })
            }
        };

        match item.body {
            FormItemBody::Checkbox(_) => {}
            _ => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::MismatchedConditionType {
                        provenance,
                        id: target_id,
                    },
                })
            }
        }

        if !self.boxes.contains(&checkbox_id) {
            return Err(FromItemsError {
                kind: FromItemsErrorKind::UnknownCheckboxIdInConditions {
                    provenance,
                    id: checkbox_id,
                },
            });
        }

        Ok(())
    }

    fn check_radio_condition(
        &mut self,
        provenance: FormItemId,
        target_id: FormItemId,
        radio_id: radio::RadioId,
    ) -> Result<(), FromItemsError> {
        let item = match self.items.get(&target_id) {
            Some(item) => item,
            None => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::UnknownFormItemIdInConditions {
                        provenance,
                        id: target_id,
                    },
                })
            }
        };

        match item.body {
            FormItemBody::Radio(_) => {}
            _ => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::MismatchedConditionType {
                        provenance,
                        id: target_id,
                    },
                })
            }
        }

        if !self.buttons.contains(&radio_id) {
            return Err(FromItemsError {
                kind: FromItemsErrorKind::UnknownRadioIdInConditions {
                    provenance,
                    id: radio_id,
                },
            });
        }

        Ok(())
    }

    fn check_grid_radio_condition(
        &mut self,
        provenance: FormItemId,
        target_id: FormItemId,
        column_id: grid_radio::GridRadioColumnId,
    ) -> Result<(), FromItemsError> {
        let item = match self.items.get(&target_id) {
            Some(item) => item,
            None => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::UnknownFormItemIdInConditions {
                        provenance,
                        id: target_id,
                    },
                })
            }
        };

        match item.body {
            FormItemBody::GridRadio(_) => {}
            _ => {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::MismatchedConditionType {
                        provenance,
                        id: target_id,
                    },
                })
            }
        }

        if !self.grid_columns.contains(&column_id) {
            return Err(FromItemsError {
                kind: FromItemsErrorKind::UnknownGridRadioColumnIdInConditions {
                    provenance,
                    id: column_id,
                },
            });
        }

        Ok(())
    }

    fn check_body(&mut self, body: &FormItemBody) -> Result<(), FromItemsError> {
        match body {
            FormItemBody::Text(_) | FormItemBody::Integer(_) => Ok(()),
            FormItemBody::Checkbox(item) => self.check_checkbox_item(&item),
            FormItemBody::Radio(item) => self.check_radio_item(&item),
            FormItemBody::GridRadio(item) => self.check_grid_radio_item(&item),
        }
    }

    fn check_checkbox_item(&mut self, item: &CheckboxFormItem) -> Result<(), FromItemsError> {
        for checkbox in item.boxes() {
            if !self.boxes.insert(checkbox.id) {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::DuplicatedCheckboxId(checkbox.id),
                });
            }
        }

        Ok(())
    }

    fn check_radio_item(&mut self, item: &RadioFormItem) -> Result<(), FromItemsError> {
        for button in item.buttons.buttons() {
            if !self.buttons.insert(button.id) {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::DuplicatedRadioId(button.id),
                });
            }
        }

        Ok(())
    }

    fn check_grid_radio_item(&mut self, item: &GridRadioFormItem) -> Result<(), FromItemsError> {
        for row in item.rows() {
            if !self.grid_rows.insert(row.id) {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::DuplicatedGridRadioRowId(row.id),
                });
            }
        }

        for column in item.columns() {
            if !self.grid_columns.insert(column.id) {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::DuplicatedGridRadioColumnId(column.id),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        radio::RadioId, CheckFormItems, FormItemCondition, FormItemId, FromItemsErrorKind,
    };
    use crate::test::model as test_model;
    use uuid::Uuid;

    #[test]
    fn test_pass() {
        assert!(matches!(
            CheckFormItems::default()
                .check_items(&[test_model::new_form_item(), test_model::new_form_item(),]),
            Ok(())
        ));
    }

    #[test]
    fn test_duplicate_item() {
        let item = test_model::new_form_item();
        assert_eq!(
            CheckFormItems::default()
                .check_items(vec![&item, &item])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::DuplicatedFormItemId(item.id)
        );
    }

    #[test]
    fn test_duplicate_radio_id() {
        let button = test_model::new_form_radio_button();
        let body = test_model::new_radio_form_item_body_with_button(button.clone());
        assert_eq!(
            CheckFormItems::default()
                .check_items(&[
                    test_model::new_form_item_with_body(body.clone()),
                    test_model::new_form_item_with_body(body),
                ])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::DuplicatedRadioId(button.id)
        );
    }

    #[test]
    fn test_unknown_item_id() {
        let button = test_model::new_form_radio_button();
        let body = test_model::new_radio_form_item_body_with_button(button.clone());
        let item = test_model::new_form_item_with_body(body.clone());

        let item_id = FormItemId::from_uuid(Uuid::new_v4());
        let condition = FormItemCondition::RadioSelected {
            item_id,
            radio_id: button.id,
        };
        let dangling_item = test_model::new_form_item_with_condition(condition);
        assert_eq!(
            CheckFormItems::default()
                .check_items(&[item, dangling_item.clone()])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::UnknownFormItemIdInConditions {
                provenance: dangling_item.id,
                id: item_id
            }
        );
    }

    #[test]
    fn test_unknown_radio_id() {
        let item = test_model::new_form_item();
        let radio_id = RadioId::from_uuid(Uuid::new_v4());
        let condition = FormItemCondition::RadioSelected {
            item_id: item.id,
            radio_id,
        };
        let dangling_item = test_model::new_form_item_with_condition(condition);
        assert_eq!(
            CheckFormItems::default()
                .check_items(&[item, dangling_item.clone()])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::UnknownRadioIdInConditions {
                provenance: dangling_item.id,
                id: radio_id
            }
        );
    }

    #[test]
    fn test_mismatched_types() {
        use super::checkbox::CheckboxId;

        let item = test_model::new_form_item_with_body(test_model::new_radio_form_item_body());
        let condition = FormItemCondition::Checkbox {
            item_id: item.id,
            checkbox_id: CheckboxId::from_uuid(Uuid::new_v4()),
            expected: true,
        };
        let bad_item = test_model::new_form_item_with_condition(condition);
        assert_eq!(
            CheckFormItems::default()
                .check_items(&[item.clone(), bad_item.clone()])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::MismatchedConditionType {
                provenance: bad_item.id,
                id: item.id
            }
        );
    }
}
