use crate::model::{
    date_time::DateTime,
    file_sharing::FileSharingId,
    form::{
        item::{
            radio::RadioId, CheckboxFormItem, FileFormItem, FormItem, FormItemBody, FormItems,
            GridRadioFormItem, IntegerFormItem, RadioFormItem, TextFormItem,
        },
        Form,
    },
    form_answer::{
        item::{
            FileSharingAnswer, FormAnswerItem, FormAnswerItemBody, FormAnswerItemChecks,
            FormAnswerItemFileSharings, FormAnswerItemGridRows, FormAnswerItemText,
            FormAnswerItems, GridRadioRowAnswer,
        },
        FormAnswer, FormAnswerContent, FormAnswerId,
    },
    project::Project,
    user::UserId,
};
use uuid::Uuid;

pub fn new_form_answer_id() -> FormAnswerId {
    FormAnswerId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_answer_item_text(item: &TextFormItem) -> Option<FormAnswerItemText> {
    let text = if let Some(min_length) = item.min_length() {
        let text = std::iter::repeat('テ')
            .take(min_length as usize)
            .collect::<String>();
        FormAnswerItemText::from_string(text).unwrap()
    } else {
        FormAnswerItemText::from_string("ア").unwrap()
    };
    Some(text)
}

pub fn mock_form_answer_item_integer(item: &IntegerFormItem) -> Option<u64> {
    if let Some(min) = item.min_limit() {
        Some(min)
    } else {
        Some(0)
    }
}

pub fn mock_form_answer_item_checkbox(item: &CheckboxFormItem) -> FormAnswerItemChecks {
    if let Some(min_checks) = item.min_checks() {
        FormAnswerItemChecks::from_checked_ids(
            item.boxes()
                .map(|checkbox| checkbox.id)
                .take(min_checks as usize),
        )
        .unwrap()
    } else {
        FormAnswerItemChecks::from_checked_ids(std::iter::empty()).unwrap()
    }
}

pub fn mock_form_answer_item_radio(item: &RadioFormItem) -> Option<RadioId> {
    Some(item.buttons.buttons().next().unwrap().id)
}

pub fn mock_form_answer_item_grid_radio(item: &GridRadioFormItem) -> FormAnswerItemGridRows {
    let answers: Vec<_> = if item.exclusive_column() {
        item.rows()
            .zip(item.columns())
            .map(|(row, column)| GridRadioRowAnswer {
                row_id: row.id,
                value: Some(column.id),
            })
            .collect()
    } else {
        let column_id = item.columns().next().unwrap().id;
        item.rows()
            .map(|row| GridRadioRowAnswer {
                row_id: row.id,
                value: Some(column_id),
            })
            .collect()
    };
    FormAnswerItemGridRows::from_row_answers(answers).unwrap()
}

pub fn mock_form_answer_item_file(item: &FileFormItem) -> FormAnswerItemFileSharings {
    let answers = if item.is_required {
        let type_ = item
            .types
            .as_ref()
            .map(|types| types.first().clone())
            .unwrap_or_default();
        let sharing_id = FileSharingId::from_uuid(Uuid::new_v4());
        let sharing_answer = FileSharingAnswer { sharing_id, type_ };
        vec![sharing_answer]
    } else {
        Vec::new()
    };
    FormAnswerItemFileSharings::from_sharing_answers(answers).unwrap()
}

pub fn mock_form_answer_item_body(body: &FormItemBody) -> FormAnswerItemBody {
    match body {
        FormItemBody::Text(item) => FormAnswerItemBody::Text(mock_form_answer_item_text(item)),
        FormItemBody::Integer(item) => {
            FormAnswerItemBody::Integer(mock_form_answer_item_integer(item))
        }
        FormItemBody::Checkbox(item) => {
            FormAnswerItemBody::Checkbox(mock_form_answer_item_checkbox(item))
        }
        FormItemBody::Radio(item) => FormAnswerItemBody::Radio(mock_form_answer_item_radio(item)),
        FormItemBody::GridRadio(item) => {
            FormAnswerItemBody::GridRadio(mock_form_answer_item_grid_radio(item))
        }
        FormItemBody::File(item) => FormAnswerItemBody::File(mock_form_answer_item_file(item)),
    }
}

pub fn mock_form_answer_item(item: &FormItem) -> FormAnswerItem {
    FormAnswerItem {
        item_id: item.id,
        body: Some(mock_form_answer_item_body(&item.body)),
    }
}

pub fn mock_form_answer_items(items: &FormItems) -> FormAnswerItems {
    FormAnswerItems::from_items(items.items().map(mock_form_answer_item)).unwrap()
}

pub fn new_form_answer_with_items(
    author_id: UserId,
    project: &Project,
    form: &Form,
    items: FormAnswerItems,
) -> FormAnswer {
    form.items().check_answer(&items).unwrap().unwrap();
    FormAnswer::from_content(FormAnswerContent {
        id: new_form_answer_id(),
        project_id: project.id(),
        form_id: form.id(),
        created_at: DateTime::now(),
        author_id,
        items,
    })
}

pub fn new_form_answer(author_id: UserId, project: &Project, form: &Form) -> FormAnswer {
    new_form_answer_with_items(
        author_id,
        project,
        form,
        mock_form_answer_items(form.items()),
    )
}
