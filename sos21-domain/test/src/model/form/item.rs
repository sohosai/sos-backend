use sos21_domain::model::form::item::{
    radio::{Radio, RadioFormItemButtons, RadioId, RadioLabel},
    FormItem, FormItemBody, FormItemDescription, FormItemId, FormItemName, FormItems,
    RadioFormItem,
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

pub fn new_form_item_body() -> FormItemBody {
    FormItemBody::Radio(RadioFormItem {
        buttons: RadioFormItemButtons::from_buttons(vec![new_form_radio_button()]).unwrap(),
        is_required: true,
    })
}

pub fn new_form_item() -> FormItem {
    FormItem {
        id: new_form_item_id(),
        name: mock_form_item_name(),
        description: mock_form_item_description(),
        conditions: None,
        body: new_form_item_body(),
    }
}

pub fn new_form_items() -> FormItems {
    FormItems::from_items(vec![new_form_item()]).unwrap()
}
