use crate::handler::model::file::FileId;
use crate::handler::model::file_sharing::FileSharingId;
use crate::handler::model::form::item::{
    CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId,
};

use serde::{Deserialize, Serialize};
use sos21_use_case::interface;
use sos21_use_case::model::form_answer::item as use_case;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioRowAnswer {
    pub row_id: GridRadioRowId,
    pub value: Option<GridRadioColumnId>,
}

impl GridRadioRowAnswer {
    pub fn from_use_case(answer: use_case::GridRadioRowAnswer) -> Self {
        GridRadioRowAnswer {
            row_id: GridRadioRowId::from_use_case(answer.row_id),
            value: answer.value.map(GridRadioColumnId::from_use_case),
        }
    }

    pub fn into_use_case(self) -> use_case::GridRadioRowAnswer {
        use_case::GridRadioRowAnswer {
            row_id: self.row_id.into_use_case(),
            value: self.value.map(GridRadioColumnId::into_use_case),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "answer")]
pub enum FormAnswerItemBody {
    Text(Option<String>),
    Integer(Option<u64>),
    Checkbox(Vec<CheckboxId>),
    Radio(Option<RadioId>),
    GridRadio(Vec<GridRadioRowAnswer>),
    File(Vec<FileSharingId>),
}

impl FormAnswerItemBody {
    pub fn from_use_case(body: use_case::FormAnswerItemBody) -> Self {
        match body {
            use_case::FormAnswerItemBody::Text(answer) => FormAnswerItemBody::Text(answer),
            use_case::FormAnswerItemBody::Integer(answer) => FormAnswerItemBody::Integer(answer),
            use_case::FormAnswerItemBody::Checkbox(answer) => FormAnswerItemBody::Checkbox(
                answer.into_iter().map(CheckboxId::from_use_case).collect(),
            ),
            use_case::FormAnswerItemBody::Radio(answer) => {
                FormAnswerItemBody::Radio(answer.map(RadioId::from_use_case))
            }
            use_case::FormAnswerItemBody::GridRadio(answer) => FormAnswerItemBody::GridRadio(
                answer
                    .into_iter()
                    .map(GridRadioRowAnswer::from_use_case)
                    .collect(),
            ),
            use_case::FormAnswerItemBody::File(answer) => FormAnswerItemBody::File(
                answer
                    .into_iter()
                    .map(FileSharingId::from_use_case)
                    .collect(),
            ),
        }
    }

    pub fn into_use_case(self) -> use_case::FormAnswerItemBody {
        match self {
            FormAnswerItemBody::Text(answer) => use_case::FormAnswerItemBody::Text(answer),
            FormAnswerItemBody::Integer(answer) => use_case::FormAnswerItemBody::Integer(answer),
            FormAnswerItemBody::Checkbox(answer) => use_case::FormAnswerItemBody::Checkbox(
                answer.into_iter().map(CheckboxId::into_use_case).collect(),
            ),
            FormAnswerItemBody::Radio(answer) => {
                use_case::FormAnswerItemBody::Radio(answer.map(RadioId::into_use_case))
            }
            FormAnswerItemBody::GridRadio(answer) => use_case::FormAnswerItemBody::GridRadio(
                answer
                    .into_iter()
                    .map(GridRadioRowAnswer::into_use_case)
                    .collect(),
            ),
            FormAnswerItemBody::File(answer) => use_case::FormAnswerItemBody::File(
                answer
                    .into_iter()
                    .map(FileSharingId::into_use_case)
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAnswerItem {
    pub item_id: FormItemId,
    #[serde(flatten)]
    pub body: Option<FormAnswerItemBody>,
}

impl FormAnswerItem {
    pub fn from_use_case(item: use_case::FormAnswerItem) -> Self {
        FormAnswerItem {
            item_id: FormItemId::from_use_case(item.item_id),
            body: item.body.map(FormAnswerItemBody::from_use_case),
        }
    }

    pub fn into_use_case(self) -> use_case::FormAnswerItem {
        use_case::FormAnswerItem {
            item_id: self.item_id.into_use_case(),
            body: self.body.map(FormAnswerItemBody::into_use_case),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestFormAnswerItem {
    pub item_id: FormItemId,
    #[serde(flatten)]
    pub body: Option<RequestFormAnswerItemBody>,
}

impl RequestFormAnswerItem {
    pub fn into_use_case(self) -> interface::form_answer::InputFormAnswerItem {
        interface::form_answer::InputFormAnswerItem {
            item_id: self.item_id.into_use_case(),
            body: self.body.map(RequestFormAnswerItemBody::into_use_case),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "answer")]
pub enum RequestFormAnswerItemBody {
    Text(Option<String>),
    Integer(Option<u64>),
    Checkbox(Vec<CheckboxId>),
    Radio(Option<RadioId>),
    GridRadio(Vec<GridRadioRowAnswer>),
    File(Vec<RequestFormAnswerItemFile>),
}

impl RequestFormAnswerItemBody {
    pub fn into_use_case(self) -> interface::form_answer::InputFormAnswerItemBody {
        match self {
            RequestFormAnswerItemBody::Text(answer) => {
                interface::form_answer::InputFormAnswerItemBody::Text(answer)
            }
            RequestFormAnswerItemBody::Integer(answer) => {
                interface::form_answer::InputFormAnswerItemBody::Integer(answer)
            }
            RequestFormAnswerItemBody::Checkbox(answer) => {
                interface::form_answer::InputFormAnswerItemBody::Checkbox(
                    answer.into_iter().map(CheckboxId::into_use_case).collect(),
                )
            }
            RequestFormAnswerItemBody::Radio(answer) => {
                interface::form_answer::InputFormAnswerItemBody::Radio(
                    answer.map(RadioId::into_use_case),
                )
            }
            RequestFormAnswerItemBody::GridRadio(answer) => {
                interface::form_answer::InputFormAnswerItemBody::GridRadio(
                    answer
                        .into_iter()
                        .map(GridRadioRowAnswer::into_use_case)
                        .collect(),
                )
            }
            RequestFormAnswerItemBody::File(answer) => {
                interface::form_answer::InputFormAnswerItemBody::File(
                    answer
                        .into_iter()
                        .map(RequestFormAnswerItemFile::into_use_case)
                        .collect(),
                )
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestFormAnswerItemFile {
    SharingId(FileSharingId),
    FileId(FileId),
}

impl RequestFormAnswerItemFile {
    pub fn into_use_case(self) -> interface::form_answer::InputFormAnswerItemFile {
        match self {
            RequestFormAnswerItemFile::FileId(file_id) => {
                interface::form_answer::InputFormAnswerItemFile::File(file_id.into_use_case())
            }
            RequestFormAnswerItemFile::SharingId(sharing_id) => {
                interface::form_answer::InputFormAnswerItemFile::Sharing(sharing_id.into_use_case())
            }
        }
    }
}
