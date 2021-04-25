use crate::interface::form_answer::{
    InputFormAnswerItem, InputFormAnswerItemBody, InputFormAnswerItemFile,
};
use crate::model::file_sharing::FileSharingId;
use crate::model::form::item::{CheckboxId, FormItemId, RadioId};
use crate::model::form_answer::item::GridRadioRowAnswer;

use sos21_domain::model::form::item;
use sos21_domain::test::model as test_model;

pub fn mock_input_form_answer_item_body(body: &item::FormItemBody) -> InputFormAnswerItemBody {
    match body {
        item::FormItemBody::Text(item) => InputFormAnswerItemBody::Text(
            test_model::mock_form_answer_item_text(item).map(|text| text.into_string()),
        ),
        item::FormItemBody::Integer(item) => {
            InputFormAnswerItemBody::Integer(test_model::mock_form_answer_item_integer(item))
        }
        item::FormItemBody::Checkbox(item) => InputFormAnswerItemBody::Checkbox(
            test_model::mock_form_answer_item_checkbox(item)
                .checked_ids()
                .map(CheckboxId::from_entity)
                .collect(),
        ),
        item::FormItemBody::Radio(item) => InputFormAnswerItemBody::Radio(
            test_model::mock_form_answer_item_radio(item).map(RadioId::from_entity),
        ),
        item::FormItemBody::GridRadio(item) => InputFormAnswerItemBody::GridRadio(
            test_model::mock_form_answer_item_grid_radio(item)
                .into_row_answers()
                .map(GridRadioRowAnswer::from_entity)
                .collect(),
        ),
        item::FormItemBody::File(item) => InputFormAnswerItemBody::File(
            test_model::mock_form_answer_item_file(item)
                .sharing_answers()
                .map(|answer| {
                    InputFormAnswerItemFile::Sharing(FileSharingId::from_entity(answer.sharing_id))
                })
                .collect(),
        ),
    }
}

pub fn mock_input_form_answer_item(item: &item::FormItem) -> InputFormAnswerItem {
    InputFormAnswerItem {
        item_id: FormItemId::from_entity(item.id),
        body: Some(mock_input_form_answer_item_body(&item.body)),
    }
}

pub fn mock_input_form_answer_items(items: &item::FormItems) -> Vec<InputFormAnswerItem> {
    items.items().map(mock_input_form_answer_item).collect()
}
