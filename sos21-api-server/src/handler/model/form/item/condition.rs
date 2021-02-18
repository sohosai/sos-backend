use serde::{Deserialize, Serialize};
use sos21_use_case::model::form::item as use_case;

use super::{CheckboxId, FormItemId, GridRadioColumnId, RadioId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
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
    pub fn from_use_case(condition: use_case::FormItemCondition) -> Self {
        match condition {
            use_case::FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected,
            } => FormItemCondition::Checkbox {
                item_id: FormItemId::from_use_case(item_id),
                checkbox_id: CheckboxId::from_use_case(checkbox_id),
                expected,
            },
            use_case::FormItemCondition::Radio {
                item_id,
                radio_id,
                expected,
            } => FormItemCondition::Radio {
                item_id: FormItemId::from_use_case(item_id),
                radio_id: RadioId::from_use_case(radio_id),
                expected,
            },
            use_case::FormItemCondition::GridRadio {
                item_id,
                column_id,
                expected,
            } => FormItemCondition::GridRadio {
                item_id: FormItemId::from_use_case(item_id),
                column_id: GridRadioColumnId::from_use_case(column_id),
                expected,
            },
        }
    }

    pub fn into_use_case(self) -> use_case::FormItemCondition {
        match self {
            FormItemCondition::Checkbox {
                item_id,
                checkbox_id,
                expected,
            } => use_case::FormItemCondition::Checkbox {
                item_id: item_id.into_use_case(),
                checkbox_id: checkbox_id.into_use_case(),
                expected,
            },
            FormItemCondition::Radio {
                item_id,
                radio_id,
                expected,
            } => use_case::FormItemCondition::Radio {
                item_id: item_id.into_use_case(),
                radio_id: radio_id.into_use_case(),
                expected,
            },
            FormItemCondition::GridRadio {
                item_id,
                column_id,
                expected,
            } => use_case::FormItemCondition::GridRadio {
                item_id: item_id.into_use_case(),
                column_id: column_id.into_use_case(),
                expected,
            },
        }
    }
}
