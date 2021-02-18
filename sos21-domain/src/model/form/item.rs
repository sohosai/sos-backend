use std::collections::{HashMap, HashSet};

use crate::model::collection::{self, LengthBoundedVec};

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn into_items(self) -> impl Iterator<Item = FormItem> {
        self.0.into_inner().into_iter()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormItemBody {
    Text(TextFormItem),
    Integer(IntegerFormItem),
    Checkbox(CheckboxFormItem),
    Radio(RadioFormItem),
    GridRadio(GridRadioFormItem),
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
            FormItemCondition::Radio {
                item_id,
                radio_id,
                expected: _,
            } => self.check_radio_condition(provenance, *item_id, *radio_id),
            FormItemCondition::GridRadio {
                item_id,
                column_id,
                expected: _,
            } => self.check_grid_radio_condition(provenance, *item_id, *column_id),
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
        for row in item.rows.rows() {
            if !self.grid_rows.insert(row.id) {
                return Err(FromItemsError {
                    kind: FromItemsErrorKind::DuplicatedGridRadioRowId(row.id),
                });
            }
        }

        for column in item.columns.columns() {
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
        radio::{Radio, RadioFormItemButtons, RadioId, RadioLabel},
        CheckFormItems, FormItem, FormItemBody, FormItemCondition, FormItemConditions,
        FormItemDescription, FormItemId, FormItemName, FromItemsErrorKind, RadioFormItem,
    };
    use sos21_domain_test::model as test_model;
    use uuid::Uuid;

    // TODO: can't we use sos21_domain_test here directly? related: rust-lang/rust#79381

    fn new_form_item_id() -> FormItemId {
        FormItemId::from_uuid(Uuid::new_v4())
    }

    fn new_form_item_with_body(body: FormItemBody) -> FormItem {
        FormItem {
            id: new_form_item_id(),
            name: FormItemName::from_string(test_model::mock_form_item_name().into_string())
                .unwrap(),
            description: FormItemDescription::from_string(
                test_model::mock_form_item_description().into_string(),
            )
            .unwrap(),
            conditions: None,
            body,
        }
    }

    fn new_form_item_with_condition(condition: FormItemCondition) -> FormItem {
        FormItem {
            id: new_form_item_id(),
            name: FormItemName::from_string(test_model::mock_form_item_name().into_string())
                .unwrap(),
            description: FormItemDescription::from_string(
                test_model::mock_form_item_description().into_string(),
            )
            .unwrap(),
            conditions: Some(FormItemConditions::from_conjunctions(vec![vec![condition]]).unwrap()),
            body: new_radio_form_item_body(),
        }
    }

    fn new_form_item() -> FormItem {
        new_form_item_with_body(new_radio_form_item_body())
    }

    pub fn new_form_radio_button() -> Radio {
        Radio {
            id: RadioId::from_uuid(Uuid::new_v4()),
            label: RadioLabel::from_string("ボタン").unwrap(),
        }
    }

    fn new_radio_form_item_body_with_button(button: Radio) -> FormItemBody {
        FormItemBody::Radio(RadioFormItem {
            buttons: RadioFormItemButtons::from_buttons(vec![button]).unwrap(),
            is_required: true,
        })
    }

    fn new_radio_form_item_body() -> FormItemBody {
        new_radio_form_item_body_with_button(new_form_radio_button())
    }

    #[test]
    fn test_pass() {
        assert!(matches!(
            CheckFormItems::default().check_items(&[new_form_item(), new_form_item(),]),
            Ok(())
        ));
    }

    #[test]
    fn test_duplicate_item() {
        let item = new_form_item();
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
        let button = new_form_radio_button();
        let body = new_radio_form_item_body_with_button(button.clone());
        assert_eq!(
            CheckFormItems::default()
                .check_items(&[
                    new_form_item_with_body(body.clone()),
                    new_form_item_with_body(body),
                ])
                .unwrap_err()
                .kind(),
            FromItemsErrorKind::DuplicatedRadioId(button.id)
        );
    }

    #[test]
    fn test_unknown_item_id() {
        let button = new_form_radio_button();
        let body = new_radio_form_item_body_with_button(button.clone());
        let item = new_form_item_with_body(body.clone());

        let item_id = FormItemId::from_uuid(Uuid::new_v4());
        let condition = FormItemCondition::Radio {
            item_id,
            radio_id: button.id,
            expected: true,
        };
        let dangling_item = new_form_item_with_condition(condition);
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
        let item = new_form_item();
        let radio_id = RadioId::from_uuid(Uuid::new_v4());
        let condition = FormItemCondition::Radio {
            item_id: item.id,
            radio_id,
            expected: true,
        };
        let dangling_item = new_form_item_with_condition(condition);
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

        let item = new_form_item_with_body(new_radio_form_item_body());
        let condition = FormItemCondition::Checkbox {
            item_id: item.id,
            checkbox_id: CheckboxId::from_uuid(Uuid::new_v4()),
            expected: true,
        };
        let bad_item = new_form_item_with_condition(condition);
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
