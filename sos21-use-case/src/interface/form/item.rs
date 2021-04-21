use crate::model::form::{
    item::{
        Checkbox, CheckboxId, FormItemBody, FormItemCondition, GridRadioColumn, GridRadioColumnId,
        GridRadioRow, GridRadioRowId, Radio, RadioId,
    },
    FormItem, FormItemId,
};

use sos21_domain::model::{
    file,
    form::{self, item},
};

#[derive(Debug, Clone)]
pub enum FormItemError {
    InvalidName,
    InvalidDescription,
    InvalidCondition,
    InvalidTextMaxLength,
    InvalidTextMinLength,
    InvalidTextPlaceholder,
    InconsistentTextLengthLimits,
    InvalidIntegerMaxLimit,
    InvalidIntegerMinLimit,
    InvalidIntegerUnit,
    OutOfLimitsIntegerPlaceholder,
    InconsistentIntegerLimits,
    InvalidCheckboxMinChecks,
    InvalidCheckboxMaxChecks,
    InvalidCheckboxLabel,
    InconsistentCheckLimits,
    NoCheckboxes,
    TooManyCheckboxes,
    InvalidRadioLabel,
    NoRadioButtons,
    TooManyRadioButtons,
    InvalidGridRadioRowLabel,
    InvalidGridRadioColumnLabel,
    NoGridRadioRows,
    TooManyGridRadioRows,
    NoGridRadioColumns,
    TooFewGridRadioColumnsWhenExclusiveAndRequired,
    TooManyGridRadioColumns,
    TooManyFileTypes,
    NoFileTypes,
    DuplicatedFileType,
    DuplicatedCheckboxId(CheckboxId),
    DuplicatedRadioId(RadioId),
    DuplicatedGridRadioRowId(GridRadioRowId),
    DuplicatedGridRadioColumnId(GridRadioColumnId),
    MismatchedConditionType(FormItemId),
    UnknownItemIdInConditions(FormItemId),
    UnknownCheckboxIdInConditions(CheckboxId),
    UnknownRadioIdInConditions(RadioId),
    UnknownGridRadioColumnIdInConditions(GridRadioColumnId),
}

impl FormItemError {
    fn from_text_content_error(err: item::text::FromContentError) -> Self {
        match err.kind() {
            item::text::FromContentErrorKind::InconsistentLengthLimits => {
                FormItemError::InconsistentTextLengthLimits
            }
        }
    }

    fn from_integer_content_error(err: item::integer::FromContentError) -> Self {
        match err.kind() {
            item::integer::FromContentErrorKind::TooBigPlaceholder => {
                FormItemError::OutOfLimitsIntegerPlaceholder
            }
            item::integer::FromContentErrorKind::TooSmallPlaceholder => {
                FormItemError::OutOfLimitsIntegerPlaceholder
            }
            item::integer::FromContentErrorKind::InconsistentLimits => {
                FormItemError::InconsistentIntegerLimits
            }
        }
    }

    fn from_checkboxes_error(err: item::checkbox::FromBoxesError) -> Self {
        match err.kind() {
            item::checkbox::FromBoxesErrorKind::Empty => FormItemError::NoCheckboxes,
            item::checkbox::FromBoxesErrorKind::TooLong => FormItemError::TooManyCheckboxes,
            item::checkbox::FromBoxesErrorKind::DuplicatedCheckboxId { id } => {
                FormItemError::DuplicatedCheckboxId(CheckboxId::from_entity(id))
            }
        }
    }

    fn from_checkbox_content_error(_err: item::checkbox::InconsistentCheckLimitsError) -> Self {
        FormItemError::InconsistentCheckLimits
    }

    fn from_grid_radio_content_error(_err: item::grid_radio::TooFewColumnsError) -> Self {
        FormItemError::TooFewGridRadioColumnsWhenExclusiveAndRequired
    }

    fn from_buttons_error(err: item::radio::FromButtonsError) -> Self {
        match err.kind() {
            item::radio::FromButtonsErrorKind::Empty => FormItemError::NoRadioButtons,
            item::radio::FromButtonsErrorKind::TooLong => FormItemError::TooManyRadioButtons,
            item::radio::FromButtonsErrorKind::DuplicatedRadioId { id } => {
                FormItemError::DuplicatedRadioId(RadioId::from_entity(id))
            }
        }
    }

    fn from_grid_rows_error(err: item::grid_radio::FromRowsError) -> Self {
        match err.kind() {
            item::grid_radio::FromRowsErrorKind::Empty => FormItemError::NoGridRadioRows,
            item::grid_radio::FromRowsErrorKind::TooLong => FormItemError::TooManyGridRadioRows,
            item::grid_radio::FromRowsErrorKind::DuplicatedRowId { id } => {
                FormItemError::DuplicatedGridRadioRowId(GridRadioRowId::from_entity(id))
            }
        }
    }

    fn from_grid_columns_error(err: item::grid_radio::FromColumnsError) -> Self {
        match err.kind() {
            item::grid_radio::FromColumnsErrorKind::Empty => FormItemError::NoGridRadioColumns,
            item::grid_radio::FromColumnsErrorKind::TooLong => {
                FormItemError::TooManyGridRadioColumns
            }
            item::grid_radio::FromColumnsErrorKind::DuplicatedColumnId { id } => {
                FormItemError::DuplicatedGridRadioColumnId(GridRadioColumnId::from_entity(id))
            }
        }
    }

    fn from_file_types_error(err: item::file::types::FromTypesError) -> Self {
        match err.kind() {
            item::file::types::FromTypesErrorKind::TooLong => FormItemError::TooManyFileTypes,
            item::file::types::FromTypesErrorKind::Empty => FormItemError::NoFileTypes,
            item::file::types::FromTypesErrorKind::Duplicated => FormItemError::DuplicatedFileType,
        }
    }
}

pub fn to_form_item(item: FormItem) -> Result<form::FormItem, FormItemError> {
    let name =
        item::FormItemName::from_string(item.name).map_err(|_| FormItemError::InvalidName)?;
    let description = item::FormItemDescription::from_string(item.description)
        .map_err(|_| FormItemError::InvalidDescription)?;
    let conditions = if let Some(conditions) = item.conditions {
        let conditions = conditions.into_iter().map(|conj| {
            conj.into_iter()
                .map(FormItemCondition::into_entity)
                .collect()
        });
        let conditions = item::FormItemConditions::from_conjunctions(conditions)
            .map_err(|_| FormItemError::InvalidCondition)?;
        Some(conditions)
    } else {
        None
    };

    let body = match item.body {
        FormItemBody::Text {
            accept_multiple_lines,
            is_required,
            max_length,
            min_length,
            placeholder,
        } => {
            let max_length = max_length
                .map(item::text::TextFormItemLength::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidTextMaxLength)?;
            let min_length = min_length
                .map(item::text::TextFormItemLength::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidTextMinLength)?;
            let placeholder = item::text::TextFormItemPlaceholder::from_string(placeholder)
                .map_err(|_| FormItemError::InvalidTextPlaceholder)?;
            let text_item = item::TextFormItem::from_content(item::text::TextFormItemContent {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            })
            .map_err(FormItemError::from_text_content_error)?;
            item::FormItemBody::Text(text_item)
        }
        FormItemBody::Integer {
            is_required,
            max,
            min,
            placeholder,
            unit,
        } => {
            let max = max
                .map(item::integer::IntegerFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidIntegerMaxLimit)?;
            let min = min
                .map(item::integer::IntegerFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidIntegerMinLimit)?;
            let unit = unit
                .map(item::integer::IntegerFormItemUnit::from_string)
                .transpose()
                .map_err(|_| FormItemError::InvalidIntegerUnit)?;
            let integer_item =
                item::IntegerFormItem::from_content(item::integer::IntegerFormItemContent {
                    is_required,
                    max,
                    min,
                    placeholder,
                    unit,
                })
                .map_err(FormItemError::from_integer_content_error)?;
            item::FormItemBody::Integer(integer_item)
        }
        FormItemBody::Checkbox {
            boxes,
            min_checks,
            max_checks,
        } => {
            let boxes = boxes
                .into_iter()
                .map(to_checkbox)
                .collect::<Result<Vec<_>, _>>()?;
            let boxes = item::checkbox::CheckboxFormItemBoxes::from_boxes(boxes)
                .map_err(FormItemError::from_checkboxes_error)?;
            let min_checks = min_checks
                .map(item::checkbox::CheckboxFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidCheckboxMinChecks)?;
            let max_checks = max_checks
                .map(item::checkbox::CheckboxFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| FormItemError::InvalidCheckboxMaxChecks)?;
            let checkbox_item =
                item::CheckboxFormItem::from_content(item::checkbox::CheckboxFormItemContent {
                    boxes,
                    min_checks,
                    max_checks,
                })
                .map_err(FormItemError::from_checkbox_content_error)?;
            item::FormItemBody::Checkbox(checkbox_item)
        }
        FormItemBody::Radio {
            buttons,
            is_required,
        } => {
            let buttons = buttons
                .into_iter()
                .map(to_radio)
                .collect::<Result<Vec<_>, _>>()?;
            let buttons = item::radio::RadioFormItemButtons::from_buttons(buttons)
                .map_err(FormItemError::from_buttons_error)?;
            let radio_item = item::RadioFormItem {
                buttons,
                is_required,
            };
            item::FormItemBody::Radio(radio_item)
        }
        FormItemBody::GridRadio {
            rows,
            columns,
            exclusive_column,
            required,
        } => {
            let rows = rows
                .into_iter()
                .map(to_grid_radio_row)
                .collect::<Result<Vec<_>, _>>()?;
            let rows = item::grid_radio::GridRadioFormItemRows::from_rows(rows)
                .map_err(FormItemError::from_grid_rows_error)?;
            let columns = columns
                .into_iter()
                .map(to_grid_radio_column)
                .collect::<Result<Vec<_>, _>>()?;
            let columns = item::grid_radio::GridRadioFormItemColumns::from_columns(columns)
                .map_err(FormItemError::from_grid_columns_error)?;
            let grid_item =
                item::GridRadioFormItem::from_content(item::grid_radio::GridRadioFormItemContent {
                    rows,
                    columns,
                    exclusive_column,
                    required: required.into_entity(),
                })
                .map_err(FormItemError::from_grid_radio_content_error)?;
            item::FormItemBody::GridRadio(grid_item)
        }
        FormItemBody::File {
            types,
            accept_multiple_files,
            is_required,
        } => {
            let types = types
                .map(|types| {
                    item::file::FileFormItemTypes::from_types(
                        types.into_iter().map(file::FileType::from_mime),
                    )
                })
                .transpose()
                .map_err(FormItemError::from_file_types_error)?;
            let file_item = item::FileFormItem {
                types,
                accept_multiple_files,
                is_required,
            };
            item::FormItemBody::File(file_item)
        }
    };

    Ok(form::FormItem {
        id: item.id.into_entity(),
        name,
        description,
        conditions,
        body,
    })
}

fn to_checkbox(checkbox: Checkbox) -> Result<item::checkbox::Checkbox, FormItemError> {
    let label = item::checkbox::CheckboxLabel::from_string(checkbox.label)
        .map_err(|_| FormItemError::InvalidCheckboxLabel)?;
    Ok(item::checkbox::Checkbox {
        id: checkbox.id.into_entity(),
        label,
    })
}

fn to_radio(radio: Radio) -> Result<item::radio::Radio, FormItemError> {
    let label = item::radio::RadioLabel::from_string(radio.label)
        .map_err(|_| FormItemError::InvalidRadioLabel)?;
    Ok(item::radio::Radio {
        id: radio.id.into_entity(),
        label,
    })
}

fn to_grid_radio_row(row: GridRadioRow) -> Result<item::grid_radio::GridRadioRow, FormItemError> {
    let label = item::grid_radio::GridRadioRowLabel::from_string(row.label)
        .map_err(|_| FormItemError::InvalidGridRadioRowLabel)?;
    Ok(item::grid_radio::GridRadioRow {
        id: row.id.into_entity(),
        label,
    })
}

fn to_grid_radio_column(
    column: GridRadioColumn,
) -> Result<item::grid_radio::GridRadioColumn, FormItemError> {
    let label = item::grid_radio::GridRadioColumnLabel::from_string(column.label)
        .map_err(|_| FormItemError::InvalidGridRadioColumnLabel)?;
    Ok(item::grid_radio::GridRadioColumn {
        id: column.id.into_entity(),
        label,
    })
}

#[derive(Debug, Clone)]
pub enum FormItemsError {
    NoItems,
    TooManyItems,
    DuplicatedItemId(FormItemId),
    InvalidItem(FormItemId, FormItemError),
}

impl FormItemsError {
    fn from_item_error(item_id: FormItemId, err: FormItemError) -> Self {
        FormItemsError::InvalidItem(item_id, err)
    }

    fn from_items_error(err: item::FromItemsError) -> Self {
        match err.kind() {
            item::FromItemsErrorKind::Empty => FormItemsError::NoItems,
            item::FromItemsErrorKind::TooLong => FormItemsError::TooManyItems,
            item::FromItemsErrorKind::DuplicatedFormItemId(id) => {
                FormItemsError::DuplicatedItemId(FormItemId::from_entity(id))
            }
            item::FromItemsErrorKind::MismatchedConditionType { provenance, id } => {
                FormItemsError::InvalidItem(
                    FormItemId::from_entity(provenance),
                    FormItemError::MismatchedConditionType(FormItemId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownFormItemIdInConditions { provenance, id } => {
                FormItemsError::InvalidItem(
                    FormItemId::from_entity(provenance),
                    FormItemError::UnknownItemIdInConditions(FormItemId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownCheckboxIdInConditions { provenance, id } => {
                FormItemsError::InvalidItem(
                    FormItemId::from_entity(provenance),
                    FormItemError::UnknownCheckboxIdInConditions(CheckboxId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownRadioIdInConditions { provenance, id } => {
                FormItemsError::InvalidItem(
                    FormItemId::from_entity(provenance),
                    FormItemError::UnknownRadioIdInConditions(RadioId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownGridRadioColumnIdInConditions { provenance, id } => {
                FormItemsError::InvalidItem(
                    FormItemId::from_entity(provenance),
                    FormItemError::UnknownGridRadioColumnIdInConditions(
                        GridRadioColumnId::from_entity(id),
                    ),
                )
            }
        }
    }
}

pub fn to_form_items<I>(items: I) -> Result<item::FormItems, FormItemsError>
where
    I: IntoIterator<Item = FormItem>,
{
    let items = items
        .into_iter()
        .map(|item| {
            let item_id = item.id;
            to_form_item(item).map_err(|err| FormItemsError::from_item_error(item_id, err))
        })
        .collect::<Result<Vec<_>, _>>()?;
    form::FormItems::from_items(items).map_err(FormItemsError::from_items_error)
}
