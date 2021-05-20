use crate::model::file_sharing::FileSharingId;
use crate::model::form::item::{
    CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId,
};

use sos21_domain::model::form_answer::item as entity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRadioRowAnswer {
    pub row_id: GridRadioRowId,
    pub value: Option<GridRadioColumnId>,
}

impl GridRadioRowAnswer {
    pub fn from_entity(answer: entity::GridRadioRowAnswer) -> Self {
        GridRadioRowAnswer {
            row_id: GridRadioRowId::from_entity(answer.row_id),
            value: answer.value.map(GridRadioColumnId::from_entity),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormAnswerItemBody {
    Text(Option<String>),
    Integer(Option<u64>),
    Checkbox(Vec<CheckboxId>),
    Radio(Option<RadioId>),
    GridRadio(Vec<GridRadioRowAnswer>),
    File(Vec<FileSharingId>),
}

impl FormAnswerItemBody {
    pub fn from_entity(body: entity::FormAnswerItemBody) -> Self {
        match body {
            entity::FormAnswerItemBody::Text(answer) => {
                FormAnswerItemBody::Text(answer.map(|t| t.into_string()))
            }
            entity::FormAnswerItemBody::Integer(answer) => FormAnswerItemBody::Integer(answer),
            entity::FormAnswerItemBody::Checkbox(answer) => {
                let checks = answer.checked_ids().map(CheckboxId::from_entity).collect();
                FormAnswerItemBody::Checkbox(checks)
            }
            entity::FormAnswerItemBody::Radio(answer) => {
                FormAnswerItemBody::Radio(answer.map(RadioId::from_entity))
            }
            entity::FormAnswerItemBody::GridRadio(answer) => {
                let answers = answer
                    .into_row_answers()
                    .map(GridRadioRowAnswer::from_entity)
                    .collect();
                FormAnswerItemBody::GridRadio(answers)
            }
            entity::FormAnswerItemBody::File(answer) => {
                let answer = answer
                    .into_sharing_answers()
                    .map(|answer| FileSharingId::from_entity(answer.sharing_id))
                    .collect();
                FormAnswerItemBody::File(answer)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormAnswerItem {
    pub item_id: FormItemId,
    pub body: Option<FormAnswerItemBody>,
}

impl FormAnswerItem {
    pub fn from_entity(item: entity::FormAnswerItem) -> Self {
        FormAnswerItem {
            item_id: FormItemId::from_entity(item.item_id),
            body: item.body.map(FormAnswerItemBody::from_entity),
        }
    }
}
