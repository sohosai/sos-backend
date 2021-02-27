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
    RadioSelected {
        item_id: FormItemId,
        radio_id: RadioId,
    },
    GridRadioSelected {
        item_id: FormItemId,
        column_id: GridRadioColumnId,
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
            use_case::FormItemCondition::RadioSelected { item_id, radio_id } => {
                FormItemCondition::RadioSelected {
                    item_id: FormItemId::from_use_case(item_id),
                    radio_id: RadioId::from_use_case(radio_id),
                }
            }
            use_case::FormItemCondition::GridRadioSelected { item_id, column_id } => {
                FormItemCondition::GridRadioSelected {
                    item_id: FormItemId::from_use_case(item_id),
                    column_id: GridRadioColumnId::from_use_case(column_id),
                }
            }
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
            FormItemCondition::RadioSelected { item_id, radio_id } => {
                use_case::FormItemCondition::RadioSelected {
                    item_id: item_id.into_use_case(),
                    radio_id: radio_id.into_use_case(),
                }
            }
            FormItemCondition::GridRadioSelected { item_id, column_id } => {
                use_case::FormItemCondition::GridRadioSelected {
                    item_id: item_id.into_use_case(),
                    column_id: column_id.into_use_case(),
                }
            }
        }
    }
}
