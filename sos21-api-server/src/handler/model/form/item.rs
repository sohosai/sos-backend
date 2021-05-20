use mime::Mime;
use serde::{Deserialize, Serialize};
use sos21_use_case::model::form::item as use_case;
use uuid::Uuid;

mod checkbox;
pub use checkbox::{Checkbox, CheckboxId};
mod radio;
pub use radio::{Radio, RadioId};
mod condition;
pub use condition::FormItemCondition;
mod grid_radio;
pub use grid_radio::{
    GridRadioColumn, GridRadioColumnId, GridRadioRequired, GridRadioRow, GridRadioRowId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormItemId(pub Uuid);

impl FormItemId {
    pub fn from_use_case(id: use_case::FormItemId) -> Self {
        FormItemId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FormItemId {
        use_case::FormItemId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FormItemBody {
    Text {
        accept_multiple_lines: bool,
        is_required: bool,
        max_length: Option<u64>,
        min_length: Option<u64>,
        placeholder: String,
    },
    Integer {
        is_required: bool,
        max: Option<u64>,
        min: Option<u64>,
        placeholder: String,
        unit: Option<String>,
    },
    Checkbox {
        boxes: Vec<Checkbox>,
        min_checks: Option<u64>,
        max_checks: Option<u64>,
    },
    Radio {
        buttons: Vec<Radio>,
        is_required: bool,
    },
    GridRadio {
        rows: Vec<GridRadioRow>,
        columns: Vec<GridRadioColumn>,
        exclusive_column: bool,
        required: GridRadioRequired,
    },
    File {
        #[serde(
            with = "crate::handler::model::serde::mime_vec_option",
            rename = "accepted_types"
        )]
        types: Option<Vec<Mime>>,
        accept_multiple_files: bool,
        is_required: bool,
    },
}

impl FormItemBody {
    pub fn from_use_case(body: use_case::FormItemBody) -> Self {
        match body {
            use_case::FormItemBody::Text {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            } => FormItemBody::Text {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            },
            use_case::FormItemBody::Integer {
                is_required,
                max,
                min,
                placeholder,
                unit,
            } => FormItemBody::Integer {
                is_required,
                max,
                min,
                placeholder,
                unit,
            },
            use_case::FormItemBody::Checkbox {
                boxes,
                min_checks,
                max_checks,
            } => FormItemBody::Checkbox {
                boxes: boxes.into_iter().map(Checkbox::from_use_case).collect(),
                min_checks,
                max_checks,
            },
            use_case::FormItemBody::Radio {
                buttons,
                is_required,
            } => FormItemBody::Radio {
                buttons: buttons.into_iter().map(Radio::from_use_case).collect(),
                is_required,
            },
            use_case::FormItemBody::GridRadio {
                rows,
                columns,
                exclusive_column,
                required,
            } => FormItemBody::GridRadio {
                rows: rows.into_iter().map(GridRadioRow::from_use_case).collect(),
                columns: columns
                    .into_iter()
                    .map(GridRadioColumn::from_use_case)
                    .collect(),
                exclusive_column,
                required: GridRadioRequired::from_use_case(required),
            },
            use_case::FormItemBody::File {
                types,
                accept_multiple_files,
                is_required,
            } => FormItemBody::File {
                types,
                accept_multiple_files,
                is_required,
            },
        }
    }

    pub fn into_use_case(self) -> use_case::FormItemBody {
        match self {
            FormItemBody::Text {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            } => use_case::FormItemBody::Text {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            },
            FormItemBody::Integer {
                is_required,
                max,
                min,
                placeholder,
                unit,
            } => use_case::FormItemBody::Integer {
                is_required,
                max,
                min,
                placeholder,
                unit,
            },
            FormItemBody::Checkbox {
                boxes,
                min_checks,
                max_checks,
            } => use_case::FormItemBody::Checkbox {
                boxes: boxes.into_iter().map(Checkbox::into_use_case).collect(),
                min_checks,
                max_checks,
            },
            FormItemBody::Radio {
                buttons,
                is_required,
            } => use_case::FormItemBody::Radio {
                buttons: buttons.into_iter().map(Radio::into_use_case).collect(),
                is_required,
            },
            FormItemBody::GridRadio {
                rows,
                columns,
                exclusive_column,
                required,
            } => use_case::FormItemBody::GridRadio {
                rows: rows.into_iter().map(GridRadioRow::into_use_case).collect(),
                columns: columns
                    .into_iter()
                    .map(GridRadioColumn::into_use_case)
                    .collect(),
                exclusive_column,
                required: required.into_use_case(),
            },
            FormItemBody::File {
                types,
                accept_multiple_files,
                is_required,
            } => use_case::FormItemBody::File {
                types,
                accept_multiple_files,
                is_required,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormItem {
    pub id: FormItemId,
    pub name: String,
    pub description: String,
    pub conditions: Option<Vec<Vec<FormItemCondition>>>,
    #[serde(flatten)]
    pub body: FormItemBody,
}

impl FormItem {
    pub fn from_use_case(item: use_case::FormItem) -> Self {
        let conditions = if let Some(conditions) = item.conditions {
            let conditions = conditions
                .into_iter()
                .map(|conj| {
                    conj.into_iter()
                        .map(FormItemCondition::from_use_case)
                        .collect()
                })
                .collect();
            Some(conditions)
        } else {
            None
        };
        FormItem {
            id: FormItemId::from_use_case(item.id),
            name: item.name,
            description: item.description,
            conditions,
            body: FormItemBody::from_use_case(item.body),
        }
    }

    pub fn into_use_case(self) -> use_case::FormItem {
        let conditions = if let Some(conditions) = self.conditions {
            let conditions = conditions
                .into_iter()
                .map(|conj| {
                    conj.into_iter()
                        .map(FormItemCondition::into_use_case)
                        .collect()
                })
                .collect();
            Some(conditions)
        } else {
            None
        };
        use_case::FormItem {
            id: self.id.into_use_case(),
            name: self.name,
            description: self.description,
            conditions,
            body: self.body.into_use_case(),
        }
    }
}
