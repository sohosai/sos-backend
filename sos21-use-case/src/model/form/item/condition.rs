use sos21_domain::model::form::item as entity;

use super::{CheckboxId, FormItemId, GridRadioColumnId, RadioId};

#[derive(Debug, Clone)]
pub enum FormItemCondition {
    Checkbox {
        item_id: FormItemId,
        checkbox_id: CheckboxId,
        expected: bool,
    },
    Radio {
        item_id: FormItemId,
        radio_id: RadioId,
        expected: bool,
    },
    GridRadio {
        item_id: FormItemId,
        column_id: GridRadioColumnId,
        expected: bool,
    },
}

impl FormItemCondition {
    pub fn from_entity(condition: entity::FormItemCondition) -> Self {
        match condition {
            entity::FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected,
            } => FormItemCondition::Checkbox {
                item_id: FormItemId::from_entity(item_id),
                checkbox_id: CheckboxId::from_entity(checkbox_id),
                expected,
            },
            entity::FormItemCondition::Radio {
                item_id,
                radio_id,
                expected,
            } => FormItemCondition::Radio {
                item_id: FormItemId::from_entity(item_id),
                radio_id: RadioId::from_entity(radio_id),
                expected,
            },
            entity::FormItemCondition::GridRadio {
                item_id,
                column_id,
                expected,
            } => FormItemCondition::GridRadio {
                item_id: FormItemId::from_entity(item_id),
                column_id: GridRadioColumnId::from_entity(column_id),
                expected,
            },
        }
    }

    pub fn into_entity(self) -> entity::FormItemCondition {
        match self {
            FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected,
            } => entity::FormItemCondition::Checkbox {
                item_id: item_id.into_entity(),
                checkbox_id: checkbox_id.into_entity(),
                expected,
            },
            FormItemCondition::Radio {
                item_id,
                radio_id,
                expected,
            } => entity::FormItemCondition::Radio {
                item_id: item_id.into_entity(),
                radio_id: radio_id.into_entity(),
                expected,
            },
            FormItemCondition::GridRadio {
                item_id,
                column_id,
                expected,
            } => entity::FormItemCondition::GridRadio {
                item_id: item_id.into_entity(),
                column_id: column_id.into_entity(),
                expected,
            },
        }
    }
}
