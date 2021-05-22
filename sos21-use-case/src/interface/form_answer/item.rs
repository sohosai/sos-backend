use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileId;
use crate::model::file_sharing::FileSharingId;
use crate::model::form::item::{
    CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId,
};
use crate::model::form_answer::item::GridRadioRowAnswer;

use anyhow::Context;
use sos21_domain::context::{FileRepository, FileSharingRepository, Login};
use sos21_domain::model::{
    file, file_sharing, form,
    form_answer::{self, item},
    pending_project, project, registration_form, registration_form_answer,
};

#[derive(Debug, Clone)]
pub struct InputFormAnswerItem {
    pub item_id: FormItemId,
    pub body: Option<InputFormAnswerItemBody>,
}

#[derive(Debug, Clone)]
pub enum InputFormAnswerItemBody {
    Text(Option<String>),
    Integer(Option<u64>),
    Checkbox(Vec<CheckboxId>),
    Radio(Option<RadioId>),
    GridRadio(Vec<GridRadioRowAnswer>),
    File(Vec<InputFormAnswerItemFile>),
}

#[derive(Debug, Clone)]
pub enum InputFormAnswerItemFile {
    Sharing(FileSharingId),
    File(FileId),
}

#[derive(Debug, Clone)]
pub enum FormAnswerItemError {
    FileNotFound,
    FileSharingNotFound,
    OutOfScopeFileSharing,
    NonSharableFile,
    TooManyFiles,
    DuplicatedFileSharingId { id: FileSharingId },
    InvalidText,
    TooManyChecks,
    NoRowAnswers,
    TooManyRowAnswers,
    DuplicatedCheckboxId { id: CheckboxId },
    DuplicatedGridRadioRowId { id: GridRadioRowId },
}

impl FormAnswerItemError {
    fn from_text_item_error(_err: item::text::TextError) -> Self {
        FormAnswerItemError::InvalidText
    }

    fn from_checks_error(err: item::checks::FromCheckedIdsError) -> Self {
        match err.kind() {
            item::checks::FromCheckedIdsErrorKind::TooLong => FormAnswerItemError::TooManyChecks,
            item::checks::FromCheckedIdsErrorKind::Duplicated(id) => {
                FormAnswerItemError::DuplicatedCheckboxId {
                    id: CheckboxId::from_entity(id),
                }
            }
        }
    }

    fn from_row_answers_error(err: item::grid_rows::FromRowAnswersError) -> Self {
        match err.kind() {
            item::grid_rows::FromRowAnswersErrorKind::Empty => FormAnswerItemError::NoRowAnswers,
            item::grid_rows::FromRowAnswersErrorKind::TooLong => {
                FormAnswerItemError::TooManyRowAnswers
            }
            item::grid_rows::FromRowAnswersErrorKind::DuplicatedRowId { id } => {
                FormAnswerItemError::DuplicatedGridRadioRowId {
                    id: GridRadioRowId::from_entity(id),
                }
            }
        }
    }

    fn from_share_error(_err: file::NonSharableFileError) -> Self {
        FormAnswerItemError::NonSharableFile
    }

    fn from_sharing_answers_error(err: item::file_sharings::FromSharingsError) -> Self {
        match err.kind() {
            item::file_sharings::FromSharingsErrorKind::TooLong => {
                FormAnswerItemError::TooManyFiles
            }
            item::file_sharings::FromSharingsErrorKind::Duplicated(id) => {
                FormAnswerItemError::DuplicatedFileSharingId {
                    id: FileSharingId::from_entity(id),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FormAnswerItemsError {
    NoItems,
    TooManyItems,
    InvalidItem(FormItemId, FormAnswerItemError),
}

impl FormAnswerItemsError {
    fn from_items_error(err: item::LengthError) -> Self {
        match err.kind() {
            item::LengthErrorKind::Empty => FormAnswerItemsError::NoItems,
            item::LengthErrorKind::TooLong => FormAnswerItemsError::TooManyItems,
        }
    }
}

pub async fn to_form_answer_items<C, I>(
    ctx: &Login<C>,
    project: &project::Project,
    form: &form::Form,
    items: I,
) -> UseCaseResult<form_answer::FormAnswerItems, FormAnswerItemsError>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
    I: IntoIterator<Item = InputFormAnswerItem>,
{
    to_form_answer_items_with_target(ctx, ShareTarget::FormAnswer { project, form }, items).await
}

pub async fn to_registration_form_answer_items<C, I>(
    ctx: &Login<C>,
    pending_project: &pending_project::PendingProject,
    registration_form: &registration_form::RegistrationForm,
    items: I,
) -> UseCaseResult<form_answer::FormAnswerItems, FormAnswerItemsError>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
    I: IntoIterator<Item = InputFormAnswerItem>,
{
    to_form_answer_items_with_target(
        ctx,
        ShareTarget::RegistrationFormAnswer {
            pending_project,
            registration_form,
        },
        items,
    )
    .await
}

/// Internal data type that carries sharing target to each functions
#[derive(Debug, Clone)]
enum ShareTarget<'a> {
    FormAnswer {
        project: &'a project::Project,
        form: &'a form::Form,
    },
    RegistrationFormAnswer {
        pending_project: &'a pending_project::PendingProject,
        registration_form: &'a registration_form::RegistrationForm,
    },
}

impl<'a> ShareTarget<'a> {
    fn to_scope(&self) -> file_sharing::FileSharingScope {
        match self {
            ShareTarget::FormAnswer { project, form } => {
                file_sharing::FileSharingScope::FormAnswer(project.id(), form.id())
            }
            ShareTarget::RegistrationFormAnswer {
                pending_project,
                registration_form,
            } => file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                    pending_project.id(),
                ),
                registration_form.id,
            ),
        }
    }

    fn is_contained_by(&self, scope: file_sharing::FileSharingScope) -> bool {
        match self {
            ShareTarget::FormAnswer { project, form } => {
                scope.contains_project_form_answer(project, form)
            }
            ShareTarget::RegistrationFormAnswer {
                pending_project,
                registration_form,
            } => scope.contains_pending_project_registration_form_answer(
                pending_project,
                registration_form,
            ),
        }
    }
}

async fn to_form_answer_items_with_target<C, I>(
    ctx: &Login<C>,
    target: ShareTarget<'_>,
    items: I,
) -> UseCaseResult<form_answer::FormAnswerItems, FormAnswerItemsError>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
    I: IntoIterator<Item = InputFormAnswerItem>,
{
    let mut result = Vec::new();
    for item in items {
        let item_id = item.item_id;
        let item = to_form_answer_item(ctx, target.clone(), item)
            .await
            .map_err(|err| {
                err.map_use_case(|err| FormAnswerItemsError::InvalidItem(item_id, err))
            })?;
        result.push(item);
    }

    form_answer::FormAnswerItems::from_items(result)
        .map_err(|err| UseCaseError::UseCase(FormAnswerItemsError::from_items_error(err)))
}

async fn to_form_answer_item<C>(
    ctx: &Login<C>,
    target: ShareTarget<'_>,
    item: InputFormAnswerItem,
) -> UseCaseResult<form_answer::FormAnswerItem, FormAnswerItemError>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
{
    let item_id = item.item_id.into_entity();

    let body = match item.body {
        Some(body) => body,
        None => {
            return Ok(form_answer::FormAnswerItem {
                item_id,
                body: None,
            })
        }
    };

    let body = match body {
        InputFormAnswerItemBody::Text(answer) => {
            let answer = answer
                .map(item::FormAnswerItemText::from_string)
                .transpose()
                .map_err(|err| {
                    UseCaseError::UseCase(FormAnswerItemError::from_text_item_error(err))
                })?;
            item::FormAnswerItemBody::Text(answer)
        }
        InputFormAnswerItemBody::Integer(answer) => item::FormAnswerItemBody::Integer(answer),
        InputFormAnswerItemBody::Checkbox(checks) => {
            let checks = checks.into_iter().map(CheckboxId::into_entity);
            let checks = item::FormAnswerItemChecks::from_checked_ids(checks).map_err(|err| {
                UseCaseError::UseCase(FormAnswerItemError::from_checks_error(err))
            })?;
            item::FormAnswerItemBody::Checkbox(checks)
        }
        InputFormAnswerItemBody::Radio(answer) => {
            item::FormAnswerItemBody::Radio(answer.map(RadioId::into_entity))
        }
        InputFormAnswerItemBody::GridRadio(rows) => {
            let rows = rows.into_iter().map(to_row_answer);
            let rows = item::FormAnswerItemGridRows::from_row_answers(rows).map_err(|err| {
                UseCaseError::UseCase(FormAnswerItemError::from_row_answers_error(err))
            })?;
            item::FormAnswerItemBody::GridRadio(rows)
        }
        InputFormAnswerItemBody::File(files) => {
            let mut sharings = Vec::new();
            for file in files {
                let sharing = to_file_sharing_answer(ctx, target.clone(), file).await?;
                sharings.push(sharing);
            }
            let sharings = item::FormAnswerItemFileSharings::from_sharing_answers(sharings)
                .map_err(|err| {
                    UseCaseError::UseCase(FormAnswerItemError::from_sharing_answers_error(err))
                })?;
            item::FormAnswerItemBody::File(sharings)
        }
    };

    Ok(form_answer::FormAnswerItem {
        item_id,
        body: Some(body),
    })
}

async fn to_file_sharing_answer<C>(
    ctx: &Login<C>,
    target: ShareTarget<'_>,
    file: InputFormAnswerItemFile,
) -> UseCaseResult<item::FileSharingAnswer, FormAnswerItemError>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    match file {
        InputFormAnswerItemFile::File(file_id) => {
            let result = ctx
                .get_file(file_id.into_entity())
                .await
                .context("Failed to get a file")?;
            let file = match result {
                Some(file) if file.is_visible_to(login_user) => file,
                _ => return Err(UseCaseError::UseCase(FormAnswerItemError::FileNotFound)),
            };

            let sharing = match file.share_by(login_user, target.to_scope()) {
                Ok(sharing) => sharing,
                Err(err) => {
                    return Err(UseCaseError::UseCase(
                        FormAnswerItemError::from_share_error(err),
                    ));
                }
            };

            ctx.store_file_sharing(sharing.clone())
                .await
                .context("Failed to store a file sharing")?;

            use_case_ensure!(target.is_contained_by(sharing.scope()));
            Ok(item::FileSharingAnswer {
                sharing_id: sharing.id(),
                type_: file.type_,
            })
        }
        InputFormAnswerItemFile::Sharing(sharing_id) => {
            let result = ctx
                .get_file_sharing(sharing_id.into_entity())
                .await
                .context("Failed to get a file sharing")?;
            let (sharing, file) = match result {
                Some((sharing, file)) if sharing.is_visible_to_with_file(login_user, &file) => {
                    (sharing, file)
                }
                _ => {
                    return Err(UseCaseError::UseCase(
                        FormAnswerItemError::FileSharingNotFound,
                    ))
                }
            };

            if !target.is_contained_by(sharing.scope()) {
                return Err(UseCaseError::UseCase(
                    FormAnswerItemError::OutOfScopeFileSharing,
                ));
            }

            Ok(item::FileSharingAnswer {
                sharing_id: sharing.id(),
                type_: file.type_,
            })
        }
    }
}

fn to_row_answer(answer: GridRadioRowAnswer) -> item::grid_rows::GridRadioRowAnswer {
    item::grid_rows::GridRadioRowAnswer {
        row_id: answer.row_id.into_entity(),
        value: answer.value.map(GridRadioColumnId::into_entity),
    }
}
