use crate::model::form::item::{
    checkbox::{Checkbox, CheckboxId, CheckboxLabel},
    grid_radio::{
        GridRadioColumn, GridRadioColumnId, GridRadioColumnLabel, GridRadioRow, GridRadioRowId,
        GridRadioRowLabel,
    },
    radio::{Radio, RadioFormItemButtons, RadioId, RadioLabel},
    FormItem, FormItemBody, FormItemCondition, FormItemConditions, FormItemDescription, FormItemId,
    FormItemName, FormItems, RadioFormItem,
};
use uuid::Uuid;

pub fn new_form_item_id() -> FormItemId {
    FormItemId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_item_name() -> FormItemName {
    FormItemName::from_string("テスト申請項目").unwrap()
}

pub fn mock_form_item_description() -> FormItemDescription {
    FormItemDescription::from_string("テスト項目").unwrap()
}

pub fn new_form_checkbox_id() -> CheckboxId {
    CheckboxId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_checkbox_label() -> CheckboxLabel {
    CheckboxLabel::from_string("ボックス").unwrap()
}

pub fn new_form_checkbox() -> Checkbox {
    Checkbox {
        id: new_form_checkbox_id(),
        label: mock_form_checkbox_label(),
    }
}
pub fn new_form_grid_radio_column_id() -> GridRadioColumnId {
    GridRadioColumnId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_grid_radio_column_label() -> GridRadioColumnLabel {
    GridRadioColumnLabel::from_string("カラム").unwrap()
}

pub fn new_form_grid_radio_column() -> GridRadioColumn {
    GridRadioColumn {
        id: new_form_grid_radio_column_id(),
        label: mock_form_grid_radio_column_label(),
    }
}

pub fn new_form_grid_radio_row_id() -> GridRadioRowId {
    GridRadioRowId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_grid_radio_row_label() -> GridRadioRowLabel {
    GridRadioRowLabel::from_string("ロウ").unwrap()
}

pub fn new_form_grid_radio_row() -> GridRadioRow {
    GridRadioRow {
        id: new_form_grid_radio_row_id(),
        label: mock_form_grid_radio_row_label(),
    }
}

pub fn new_form_radio_button_id() -> RadioId {
    RadioId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_radio_button_label() -> RadioLabel {
    RadioLabel::from_string("ボタン").unwrap()
}

pub fn new_form_radio_button() -> Radio {
    Radio {
        id: new_form_radio_button_id(),
        label: mock_form_radio_button_label(),
    }
}

pub fn new_radio_form_item_body_with_button(button: Radio) -> FormItemBody {
    FormItemBody::Radio(RadioFormItem {
        buttons: RadioFormItemButtons::from_buttons(vec![button]).unwrap(),
        is_required: true,
    })
}

pub fn new_radio_form_item_body() -> FormItemBody {
    new_radio_form_item_body_with_button(new_form_radio_button())
}

pub fn new_form_item_body() -> FormItemBody {
    new_radio_form_item_body()
}

pub fn new_form_item_with_body(body: FormItemBody) -> FormItem {
    FormItem {
        id: new_form_item_id(),
        name: mock_form_item_name(),
        description: mock_form_item_description(),
        conditions: None,
        body,
    }
}

pub fn new_form_item_with_condition(condition: FormItemCondition) -> FormItem {
    FormItem {
        id: new_form_item_id(),
        name: mock_form_item_name(),
        description: mock_form_item_description(),
        conditions: Some(FormItemConditions::from_conjunctions(vec![vec![condition]]).unwrap()),
        body: new_form_item_body(),
    }
}

pub fn new_form_item() -> FormItem {
    new_form_item_with_body(new_radio_form_item_body())
}

pub fn new_form_items() -> FormItems {
    FormItems::from_items(vec![new_form_item()]).unwrap()
}
