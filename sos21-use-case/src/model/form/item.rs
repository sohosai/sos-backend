use sos21_domain::model::form::item as entity;
use uuid::Uuid;

mod checkbox;
pub use checkbox::{Checkbox, CheckboxId};
mod condition;
pub use condition::FormItemCondition;
mod radio;
pub use radio::{Radio, RadioId};
mod grid_radio;
pub use grid_radio::{
    GridRadioColumn, GridRadioColumnId, GridRadioRequired, GridRadioRow, GridRadioRowId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormItemId(pub Uuid);

impl FormItemId {
    pub fn from_entity(id: entity::FormItemId) -> FormItemId {
        FormItemId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FormItemId {
        entity::FormItemId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
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
        placeholder: u64,
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
}

impl FormItemBody {
    pub fn from_entity(body: entity::FormItemBody) -> Self {
        match body {
            entity::FormItemBody::Text(item) => {
                let item = item.into_content();
                FormItemBody::Text {
                    accept_multiple_lines: item.accept_multiple_lines,
                    is_required: item.is_required,
                    max_length: item.max_length.map(|l| l.to_u64()),
                    min_length: item.min_length.map(|l| l.to_u64()),
                    placeholder: item.placeholder.into_string(),
                }
            }
            entity::FormItemBody::Integer(item) => {
                let item = item.into_content();
                FormItemBody::Integer {
                    is_required: item.is_required,
                    max: item.max.map(|l| l.to_u64()),
                    min: item.min.map(|l| l.to_u64()),
                    placeholder: item.placeholder,
                    unit: item.unit.map(|u| u.into_string()),
                }
            }
            entity::FormItemBody::Checkbox(item) => {
                let item = item.into_content();
                let boxes = item.boxes.into_boxes().map(Checkbox::from_entity).collect();
                FormItemBody::Checkbox {
                    boxes,
                    min_checks: item.min_checks.map(|l| l.to_u64()),
                    max_checks: item.max_checks.map(|l| l.to_u64()),
                }
            }
            entity::FormItemBody::Radio(item) => {
                let buttons = item
                    .buttons
                    .into_buttons()
                    .map(Radio::from_entity)
                    .collect();
                FormItemBody::Radio {
                    buttons,
                    is_required: item.is_required,
                }
            }
            entity::FormItemBody::GridRadio(item) => {
                let item = item.into_content();
                let rows = item
                    .rows
                    .into_rows()
                    .map(GridRadioRow::from_entity)
                    .collect();
                let columns = item
                    .columns
                    .into_columns()
                    .map(GridRadioColumn::from_entity)
                    .collect();
                FormItemBody::GridRadio {
                    rows,
                    columns,
                    required: GridRadioRequired::from_entity(item.required),
                    exclusive_column: item.exclusive_column,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormItem {
    pub id: FormItemId,
    pub name: String,
    pub description: String,
    pub conditions: Option<Vec<Vec<FormItemCondition>>>,
    pub body: FormItemBody,
}

impl FormItem {
    pub fn from_entity(item: entity::FormItem) -> Self {
        let conditions = item.conditions.map(|conditions| {
            conditions
                .into_conjunctions()
                .map(|conj| {
                    conj.into_iter()
                        .map(FormItemCondition::from_entity)
                        .collect()
                })
                .collect()
        });
        FormItem {
            id: FormItemId::from_entity(item.id),
            name: item.name.into_string(),
            description: item.description.into_string(),
            conditions,
            body: FormItemBody::from_entity(item.body),
        }
    }
}
