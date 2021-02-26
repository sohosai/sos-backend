use sos21_domain::model::{
    date_time::DateTime,
    form::{
        item::{FormItem, FormItemBody, FormItems},
        Form,
    },
    form_answer::{
        item::{
            FormAnswerItem, FormAnswerItemBody, FormAnswerItemChecks, FormAnswerItemGridRows,
            FormAnswerItemText, FormAnswerItems, GridRadioRowAnswer,
        },
        FormAnswer, FormAnswerId,
    },
    project::ProjectId,
    user::UserId,
};
use uuid::Uuid;

pub fn new_form_answer_id() -> FormAnswerId {
    FormAnswerId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_answer_item_body(body: &FormItemBody) -> FormAnswerItemBody {
    match body {
        FormItemBody::Text(item) => {
            let text = if let Some(min_length) = item.min_length() {
                let text = std::iter::repeat('テ')
                    .take(min_length as usize)
                    .collect::<String>();
                FormAnswerItemText::from_string(text).unwrap()
            } else {
                FormAnswerItemText::from_string("ア").unwrap()
            };
            FormAnswerItemBody::Text(Some(text))
        }
        FormItemBody::Integer(item) => {
            if let Some(min) = item.min_limit() {
                FormAnswerItemBody::Integer(Some(min))
            } else {
                FormAnswerItemBody::Integer(Some(0))
            }
        }
        FormItemBody::Checkbox(item) => {
            let checks = if let Some(min_checks) = item.min_checks() {
                FormAnswerItemChecks::from_checked_ids(
                    item.boxes()
                        .map(|checkbox| checkbox.id)
                        .take(min_checks as usize),
                )
                .unwrap()
            } else {
                FormAnswerItemChecks::from_checked_ids(std::iter::empty()).unwrap()
            };
            FormAnswerItemBody::Checkbox(checks)
        }
        FormItemBody::Radio(item) => {
            FormAnswerItemBody::Radio(Some(item.buttons.buttons().next().unwrap().id))
        }
        FormItemBody::GridRadio(item) => {
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
            FormAnswerItemBody::GridRadio(
                FormAnswerItemGridRows::from_row_answers(answers).unwrap(),
            )
        }
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

pub fn new_form_answer(author_id: UserId, project_id: ProjectId, form: &Form) -> FormAnswer {
    FormAnswer {
        id: new_form_answer_id(),
        project_id,
        form_id: form.id,
        created_at: DateTime::now(),
        author_id,
        items: mock_form_answer_items(&form.items),
    }
}
