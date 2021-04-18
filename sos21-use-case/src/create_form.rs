use crate::error::{UseCaseError, UseCaseResult};
use crate::model::form::{
    item::{
        Checkbox, CheckboxId, FormItemBody, FormItemCondition, GridRadioColumn, GridRadioColumnId,
        GridRadioRow, GridRadioRowId, Radio, RadioId,
    },
    Form, FormCondition, FormItem, FormItemId,
};
use crate::model::project::{ProjectAttribute, ProjectCategory, ProjectId};
use crate::model::project_query::{ProjectQuery, ProjectQueryConjunction};

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login};
use sos21_domain::model::permissions::Permissions;
use sos21_domain::model::{
    date_time::DateTime,
    file,
    form::{self, item},
    project, project_query,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub description: String,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: chrono::DateTime<chrono::Utc>,
    pub items: Vec<FormItem>,
    pub condition: FormCondition,
}

#[derive(Debug, Clone)]
pub enum ItemError {
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

impl ItemError {
    fn from_text_content_error(err: item::text::FromContentError) -> Self {
        match err.kind() {
            item::text::FromContentErrorKind::InconsistentLengthLimits => {
                ItemError::InconsistentTextLengthLimits
            }
        }
    }

    fn from_integer_content_error(err: item::integer::FromContentError) -> Self {
        match err.kind() {
            item::integer::FromContentErrorKind::TooBigPlaceholder => {
                ItemError::OutOfLimitsIntegerPlaceholder
            }
            item::integer::FromContentErrorKind::TooSmallPlaceholder => {
                ItemError::OutOfLimitsIntegerPlaceholder
            }
            item::integer::FromContentErrorKind::InconsistentLimits => {
                ItemError::InconsistentIntegerLimits
            }
        }
    }

    fn from_checkboxes_error(err: item::checkbox::FromBoxesError) -> Self {
        match err.kind() {
            item::checkbox::FromBoxesErrorKind::Empty => ItemError::NoCheckboxes,
            item::checkbox::FromBoxesErrorKind::TooLong => ItemError::TooManyCheckboxes,
            item::checkbox::FromBoxesErrorKind::DuplicatedCheckboxId { id } => {
                ItemError::DuplicatedCheckboxId(CheckboxId::from_entity(id))
            }
        }
    }

    fn from_checkbox_content_error(_err: item::checkbox::InconsistentCheckLimitsError) -> Self {
        ItemError::InconsistentCheckLimits
    }

    fn from_grid_radio_content_error(_err: item::grid_radio::TooFewColumnsError) -> Self {
        ItemError::TooFewGridRadioColumnsWhenExclusiveAndRequired
    }

    fn from_buttons_error(err: item::radio::FromButtonsError) -> Self {
        match err.kind() {
            item::radio::FromButtonsErrorKind::Empty => ItemError::NoRadioButtons,
            item::radio::FromButtonsErrorKind::TooLong => ItemError::TooManyRadioButtons,
            item::radio::FromButtonsErrorKind::DuplicatedRadioId { id } => {
                ItemError::DuplicatedRadioId(RadioId::from_entity(id))
            }
        }
    }

    fn from_grid_rows_error(err: item::grid_radio::FromRowsError) -> Self {
        match err.kind() {
            item::grid_radio::FromRowsErrorKind::Empty => ItemError::NoGridRadioRows,
            item::grid_radio::FromRowsErrorKind::TooLong => ItemError::TooManyGridRadioRows,
            item::grid_radio::FromRowsErrorKind::DuplicatedRowId { id } => {
                ItemError::DuplicatedGridRadioRowId(GridRadioRowId::from_entity(id))
            }
        }
    }

    fn from_grid_columns_error(err: item::grid_radio::FromColumnsError) -> Self {
        match err.kind() {
            item::grid_radio::FromColumnsErrorKind::Empty => ItemError::NoGridRadioColumns,
            item::grid_radio::FromColumnsErrorKind::TooLong => ItemError::TooManyGridRadioColumns,
            item::grid_radio::FromColumnsErrorKind::DuplicatedColumnId { id } => {
                ItemError::DuplicatedGridRadioColumnId(GridRadioColumnId::from_entity(id))
            }
        }
    }

    fn from_file_types_error(err: item::file::types::FromTypesError) -> Self {
        match err.kind() {
            item::file::types::FromTypesErrorKind::TooLong => ItemError::TooManyFileTypes,
            item::file::types::FromTypesErrorKind::Empty => ItemError::NoFileTypes,
            item::file::types::FromTypesErrorKind::Duplicated => ItemError::DuplicatedFileType,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConditionError {
    TooBigQuery,
    DuplicatedAttributes,
    TooManyIncludeProjects,
    DuplicatedIncludeProjects,
    TooManyExcludeProjects,
    DuplicatedExcludeProjects,
}

impl ConditionError {
    fn from_query_error(err: project_query::FromConjunctionsError) -> Self {
        match err.kind() {
            project_query::FromConjunctionsErrorKind::TooBigDisjunction => {
                ConditionError::TooBigQuery
            }
        }
    }

    fn from_attributes_error(_err: project::attribute::DuplicatedAttributesError) -> Self {
        ConditionError::DuplicatedAttributes
    }

    fn from_includes_error(err: form::condition::FromProjectsError) -> Self {
        match err.kind() {
            form::condition::FromProjectsErrorKind::TooLong => {
                ConditionError::TooManyIncludeProjects
            }
            form::condition::FromProjectsErrorKind::Duplicated(_) => {
                ConditionError::DuplicatedIncludeProjects
            }
        }
    }

    fn from_excludes_error(err: form::condition::FromProjectsError) -> Self {
        match err.kind() {
            form::condition::FromProjectsErrorKind::TooLong => {
                ConditionError::TooManyExcludeProjects
            }
            form::condition::FromProjectsErrorKind::Duplicated(_) => {
                ConditionError::DuplicatedExcludeProjects
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    InvalidDescription,
    InvalidPeriod,
    NoItems,
    TooManyItems,
    DuplicatedItemId(FormItemId),
    InvalidItem(FormItemId, ItemError),
    InvalidCondition(ConditionError),
    InsufficientPermissions,
}

impl Error {
    fn from_items_error(err: item::FromItemsError) -> Self {
        match err.kind() {
            item::FromItemsErrorKind::Empty => Error::NoItems,
            item::FromItemsErrorKind::TooLong => Error::TooManyItems,
            item::FromItemsErrorKind::DuplicatedFormItemId(id) => {
                Error::DuplicatedItemId(FormItemId::from_entity(id))
            }
            item::FromItemsErrorKind::MismatchedConditionType { provenance, id } => {
                Error::InvalidItem(
                    FormItemId::from_entity(provenance),
                    ItemError::MismatchedConditionType(FormItemId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownFormItemIdInConditions { provenance, id } => {
                Error::InvalidItem(
                    FormItemId::from_entity(provenance),
                    ItemError::UnknownItemIdInConditions(FormItemId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownCheckboxIdInConditions { provenance, id } => {
                Error::InvalidItem(
                    FormItemId::from_entity(provenance),
                    ItemError::UnknownCheckboxIdInConditions(CheckboxId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownRadioIdInConditions { provenance, id } => {
                Error::InvalidItem(
                    FormItemId::from_entity(provenance),
                    ItemError::UnknownRadioIdInConditions(RadioId::from_entity(id)),
                )
            }
            item::FromItemsErrorKind::UnknownGridRadioColumnIdInConditions { provenance, id } => {
                Error::InvalidItem(
                    FormItemId::from_entity(provenance),
                    ItemError::UnknownGridRadioColumnIdInConditions(
                        GridRadioColumnId::from_entity(id),
                    ),
                )
            }
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Form, Error>
where
    C: FormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::CREATE_FORMS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let name = form::FormName::from_string(input.name)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidName))?;
    let description = form::FormDescription::from_string(input.description)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidDescription))?;
    let items = input
        .items
        .into_iter()
        .map(|item| {
            let item_id = item.id;
            to_form_item(item)
                .map_err(|err| UseCaseError::UseCase(Error::InvalidItem(item_id, err)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let items = form::FormItems::from_items(items)
        .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;
    let condition = to_form_condition(input.condition)
        .map_err(|err| UseCaseError::UseCase(Error::InvalidCondition(err)))?;
    let starts_at = DateTime::from_utc(input.starts_at);
    let ends_at = DateTime::from_utc(input.ends_at);
    let period = form::FormPeriod::from_datetime(starts_at, ends_at)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidPeriod))?;

    let form = form::Form {
        id: form::FormId::from_uuid(Uuid::new_v4()),
        created_at: DateTime::now(),
        author_id: login_user.id.clone(),
        name,
        description,
        period,
        items,
        condition,
    };
    ctx.store_form(form.clone())
        .await
        .context("Failed to store a form")?;
    use_case_ensure!(form.is_visible_to(login_user));
    Ok(Form::from_entity(form))
}

fn to_form_item(item: FormItem) -> Result<form::FormItem, ItemError> {
    let name = item::FormItemName::from_string(item.name).map_err(|_| ItemError::InvalidName)?;
    let description = item::FormItemDescription::from_string(item.description)
        .map_err(|_| ItemError::InvalidDescription)?;
    let conditions = if let Some(conditions) = item.conditions {
        let conditions = conditions.into_iter().map(|conj| {
            conj.into_iter()
                .map(FormItemCondition::into_entity)
                .collect()
        });
        let conditions = item::FormItemConditions::from_conjunctions(conditions)
            .map_err(|_| ItemError::InvalidCondition)?;
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
                .map_err(|_| ItemError::InvalidTextMaxLength)?;
            let min_length = min_length
                .map(item::text::TextFormItemLength::from_u64)
                .transpose()
                .map_err(|_| ItemError::InvalidTextMinLength)?;
            let placeholder = item::text::TextFormItemPlaceholder::from_string(placeholder)
                .map_err(|_| ItemError::InvalidTextPlaceholder)?;
            let text_item = item::TextFormItem::from_content(item::text::TextFormItemContent {
                accept_multiple_lines,
                is_required,
                max_length,
                min_length,
                placeholder,
            })
            .map_err(ItemError::from_text_content_error)?;
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
                .map_err(|_| ItemError::InvalidIntegerMaxLimit)?;
            let min = min
                .map(item::integer::IntegerFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| ItemError::InvalidIntegerMinLimit)?;
            let unit = unit
                .map(item::integer::IntegerFormItemUnit::from_string)
                .transpose()
                .map_err(|_| ItemError::InvalidIntegerUnit)?;
            let integer_item =
                item::IntegerFormItem::from_content(item::integer::IntegerFormItemContent {
                    is_required,
                    max,
                    min,
                    placeholder,
                    unit,
                })
                .map_err(ItemError::from_integer_content_error)?;
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
                .map_err(ItemError::from_checkboxes_error)?;
            let min_checks = min_checks
                .map(item::checkbox::CheckboxFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| ItemError::InvalidCheckboxMinChecks)?;
            let max_checks = max_checks
                .map(item::checkbox::CheckboxFormItemLimit::from_u64)
                .transpose()
                .map_err(|_| ItemError::InvalidCheckboxMaxChecks)?;
            let checkbox_item =
                item::CheckboxFormItem::from_content(item::checkbox::CheckboxFormItemContent {
                    boxes,
                    min_checks,
                    max_checks,
                })
                .map_err(ItemError::from_checkbox_content_error)?;
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
                .map_err(ItemError::from_buttons_error)?;
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
                .map_err(ItemError::from_grid_rows_error)?;
            let columns = columns
                .into_iter()
                .map(to_grid_radio_column)
                .collect::<Result<Vec<_>, _>>()?;
            let columns = item::grid_radio::GridRadioFormItemColumns::from_columns(columns)
                .map_err(ItemError::from_grid_columns_error)?;
            let grid_item =
                item::GridRadioFormItem::from_content(item::grid_radio::GridRadioFormItemContent {
                    rows,
                    columns,
                    exclusive_column,
                    required: required.into_entity(),
                })
                .map_err(ItemError::from_grid_radio_content_error)?;
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
                .map_err(ItemError::from_file_types_error)?;
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

fn to_checkbox(checkbox: Checkbox) -> Result<item::checkbox::Checkbox, ItemError> {
    let label = item::checkbox::CheckboxLabel::from_string(checkbox.label)
        .map_err(|_| ItemError::InvalidCheckboxLabel)?;
    Ok(item::checkbox::Checkbox {
        id: checkbox.id.into_entity(),
        label,
    })
}

fn to_radio(radio: Radio) -> Result<item::radio::Radio, ItemError> {
    let label = item::radio::RadioLabel::from_string(radio.label)
        .map_err(|_| ItemError::InvalidRadioLabel)?;
    Ok(item::radio::Radio {
        id: radio.id.into_entity(),
        label,
    })
}

fn to_grid_radio_row(row: GridRadioRow) -> Result<item::grid_radio::GridRadioRow, ItemError> {
    let label = item::grid_radio::GridRadioRowLabel::from_string(row.label)
        .map_err(|_| ItemError::InvalidGridRadioRowLabel)?;
    Ok(item::grid_radio::GridRadioRow {
        id: row.id.into_entity(),
        label,
    })
}

fn to_grid_radio_column(
    column: GridRadioColumn,
) -> Result<item::grid_radio::GridRadioColumn, ItemError> {
    let label = item::grid_radio::GridRadioColumnLabel::from_string(column.label)
        .map_err(|_| ItemError::InvalidGridRadioColumnLabel)?;
    Ok(item::grid_radio::GridRadioColumn {
        id: column.id.into_entity(),
        label,
    })
}

fn to_form_condition(condition: FormCondition) -> Result<form::FormCondition, ConditionError> {
    let query = to_project_query(condition.query)?;
    let includes = form::FormConditionProjectSet::from_projects(
        condition.includes.into_iter().map(ProjectId::into_entity),
    )
    .map_err(ConditionError::from_includes_error)?;
    let excludes = form::FormConditionProjectSet::from_projects(
        condition.excludes.into_iter().map(ProjectId::into_entity),
    )
    .map_err(ConditionError::from_excludes_error)?;

    Ok(form::FormCondition {
        query,
        includes,
        excludes,
    })
}

fn to_project_query(query: ProjectQuery) -> Result<project_query::ProjectQuery, ConditionError> {
    let dnf = query
        .0
        .into_iter()
        .map(to_project_query_conjunction)
        .collect::<Result<Vec<_>, _>>()?;
    project_query::ProjectQuery::from_conjunctions(dnf).map_err(ConditionError::from_query_error)
}

fn to_project_query_conjunction(
    conj: ProjectQueryConjunction,
) -> Result<project_query::ProjectQueryConjunction, ConditionError> {
    let category = conj.category.map(ProjectCategory::into_entity);
    let attributes = project::ProjectAttributes::from_attributes(
        conj.attributes
            .into_iter()
            .map(ProjectAttribute::into_entity),
    )
    .map_err(ConditionError::from_attributes_error)?;

    Ok(project_query::ProjectQueryConjunction {
        category,
        attributes,
    })
}

#[cfg(test)]
mod tests {
    use crate::model::{
        form::{FormCondition, FormItem},
        user::UserId,
    };
    use crate::{create_form, get_form, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot create forms.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period();
        let input = create_form::Input {
            name: test::model::mock_form_name().into_string(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        assert!(matches!(
            create_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot create forms.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period();
        let input = create_form::Input {
            name: test::model::mock_form_name().into_string(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        assert!(matches!(
            create_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user can create forms.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period();
        let name = "テストテストテスト".to_string();
        let input = create_form::Input {
            name: name.clone(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        let result = create_form::run(&app, input).await;
        assert!(result.is_ok());

        let got = result.unwrap();
        assert!(got.name == name);
        assert!(got.author_id == UserId::from_entity(user.id));

        assert!(matches!(get_form::run(&app, got.id).await, Ok(_)));
    }
}
