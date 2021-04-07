use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileId;
use crate::model::file_sharing::FileSharingId;
use crate::model::form::{
    item::{CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId},
    FormId,
};
use crate::model::form_answer::{item::GridRadioRowAnswer, FormAnswer};
use crate::model::project::ProjectId;

use anyhow::{ensure, Context};
use sos21_domain::context::{
    FileRepository, FileSharingRepository, FormAnswerRepository, FormRepository, Login,
    ProjectRepository,
};
use sos21_domain::model::{
    date_time::DateTime,
    file, file_sharing, form,
    form_answer::{self, item},
    project,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub items: Vec<InputFormAnswerItem>,
}

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
pub enum ItemError {
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

impl ItemError {
    fn from_checks_error(err: item::checks::FromCheckedIdsError) -> Self {
        match err.kind() {
            item::checks::FromCheckedIdsErrorKind::TooLong => ItemError::TooManyChecks,
            item::checks::FromCheckedIdsErrorKind::Duplicated(id) => {
                ItemError::DuplicatedCheckboxId {
                    id: CheckboxId::from_entity(id),
                }
            }
        }
    }

    fn from_row_answers_error(err: item::grid_rows::FromRowAnswersError) -> Self {
        match err.kind() {
            item::grid_rows::FromRowAnswersErrorKind::Empty => ItemError::NoRowAnswers,
            item::grid_rows::FromRowAnswersErrorKind::TooLong => ItemError::TooManyRowAnswers,
            item::grid_rows::FromRowAnswersErrorKind::DuplicatedRowId { id } => {
                ItemError::DuplicatedGridRadioRowId {
                    id: GridRadioRowId::from_entity(id),
                }
            }
        }
    }

    fn from_share_error(_err: file::NonSharableFileError) -> Self {
        ItemError::NonSharableFile
    }

    fn from_sharing_answers_error(err: item::file_sharings::FromSharingsError) -> Self {
        match err.kind() {
            item::file_sharings::FromSharingsErrorKind::TooLong => ItemError::TooManyFiles,
            item::file_sharings::FromSharingsErrorKind::Duplicated(id) => {
                ItemError::DuplicatedFileSharingId {
                    id: FileSharingId::from_entity(id),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnswerError {
    NotAnsweredWithoutCondition,
    NotAnsweredWithCondition,
    UnexpectedAnswer,
    MismatchedItemType,
    NotAnsweredText,
    TooLongText,
    TooShortText,
    NotAllowedMultipleLineText,
    NotAnsweredInteger,
    TooBigInteger,
    TooSmallInteger,
    TooManyChecks,
    TooFewChecks,
    NotAnsweredFile,
    NotAllowedMultipleFiles,
    NotAllowedFileType,
    UnknownCheckboxId {
        id: CheckboxId,
    },
    NotAnsweredRadio,
    UnknownRadioId {
        id: RadioId,
    },
    NotAnsweredGridRadioRows,
    MismatchedGridRadioRowsLength,
    MismatchedGridRadioRowId {
        expected: GridRadioRowId,
        got: GridRadioRowId,
    },
    UnknownGridRadioColumnId {
        id: GridRadioColumnId,
    },
    NotAllowedDuplicatedGridRadioColumn {
        id: GridRadioColumnId,
    },
}

impl AnswerError {
    fn from_check_error_kind(kind: form::item::CheckAnswerItemErrorKind) -> Self {
        match kind {
            form::item::CheckAnswerItemErrorKind::NotAnsweredWithoutCondition => {
                AnswerError::NotAnsweredWithoutCondition
            }
            form::item::CheckAnswerItemErrorKind::NotAnsweredWithCondition => {
                AnswerError::NotAnsweredWithCondition
            }
            form::item::CheckAnswerItemErrorKind::UnexpectedAnswer => AnswerError::UnexpectedAnswer,
            form::item::CheckAnswerItemErrorKind::MismatchedItemType => {
                AnswerError::MismatchedItemType
            }
            form::item::CheckAnswerItemErrorKind::NotAnsweredText => AnswerError::NotAnsweredText,
            form::item::CheckAnswerItemErrorKind::TooLongText => AnswerError::TooLongText,
            form::item::CheckAnswerItemErrorKind::TooShortText => AnswerError::TooShortText,
            form::item::CheckAnswerItemErrorKind::NotAllowedMultipleLineText => {
                AnswerError::NotAllowedMultipleLineText
            }
            form::item::CheckAnswerItemErrorKind::NotAnsweredInteger => {
                AnswerError::NotAnsweredInteger
            }
            form::item::CheckAnswerItemErrorKind::TooBigInteger => AnswerError::TooBigInteger,
            form::item::CheckAnswerItemErrorKind::TooSmallInteger => AnswerError::TooSmallInteger,
            form::item::CheckAnswerItemErrorKind::TooManyChecks => AnswerError::TooManyChecks,
            form::item::CheckAnswerItemErrorKind::TooFewChecks => AnswerError::TooFewChecks,
            form::item::CheckAnswerItemErrorKind::NotAnsweredRadio => AnswerError::NotAnsweredRadio,
            form::item::CheckAnswerItemErrorKind::NotAnsweredGridRadioRows => {
                AnswerError::NotAnsweredGridRadioRows
            }
            form::item::CheckAnswerItemErrorKind::NotAnsweredFile => AnswerError::NotAnsweredFile,
            form::item::CheckAnswerItemErrorKind::NotAllowedMultipleFiles => {
                AnswerError::NotAllowedMultipleFiles
            }
            form::item::CheckAnswerItemErrorKind::NotAllowedFileType => {
                AnswerError::NotAllowedFileType
            }
            form::item::CheckAnswerItemErrorKind::UnknownCheckboxId { id } => {
                AnswerError::UnknownCheckboxId {
                    id: CheckboxId::from_entity(id),
                }
            }
            form::item::CheckAnswerItemErrorKind::UnknownRadioId { id } => {
                AnswerError::UnknownRadioId {
                    id: RadioId::from_entity(id),
                }
            }
            form::item::CheckAnswerItemErrorKind::MismatchedGridRadioRowsLength => {
                AnswerError::MismatchedGridRadioRowsLength
            }
            form::item::CheckAnswerItemErrorKind::MismatchedGridRadioRowId { expected, got } => {
                AnswerError::MismatchedGridRadioRowId {
                    expected: GridRadioRowId::from_entity(expected),
                    got: GridRadioRowId::from_entity(got),
                }
            }
            form::item::CheckAnswerItemErrorKind::UnknownGridRadioColumnId { id } => {
                AnswerError::UnknownGridRadioColumnId {
                    id: GridRadioColumnId::from_entity(id),
                }
            }
            form::item::CheckAnswerItemErrorKind::NotAllowedDuplicatedGridRadioColumn { id } => {
                AnswerError::NotAllowedDuplicatedGridRadioColumn {
                    id: GridRadioColumnId::from_entity(id),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FormNotFound,
    OutOfAnswerPeriod,
    AlreadyAnswered,
    NoItems,
    TooManyItems,
    InvalidItem(FormItemId, ItemError),
    MismatchedItemsLength,
    MismatchedItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    InvalidAnswer(FormItemId, AnswerError),
}

impl Error {
    fn from_items_error(err: item::LengthError) -> Self {
        match err.kind() {
            item::LengthErrorKind::Empty => Error::NoItems,
            item::LengthErrorKind::TooLong => Error::TooManyItems,
        }
    }

    fn from_check_error(err: form::item::CheckAnswerError) -> Self {
        match err.kind() {
            form::item::CheckAnswerErrorKind::MismatchedItemsLength => Error::MismatchedItemsLength,
            form::item::CheckAnswerErrorKind::MismatchedItemId { expected, got } => {
                Error::MismatchedItemId {
                    expected: FormItemId::from_entity(expected),
                    got: FormItemId::from_entity(got),
                }
            }
            form::item::CheckAnswerErrorKind::Item(item_id, inner) => Error::InvalidAnswer(
                FormItemId::from_entity(item_id),
                AnswerError::from_check_error_kind(inner),
            ),
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<FormAnswer, Error>
where
    C: ProjectRepository
        + FormRepository
        + FormAnswerRepository
        + FileRepository
        + FileSharingRepository
        + Send
        + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(input.project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    let result = ctx
        .get_form(input.form_id.into_entity())
        .await
        .context("Failed to get a form")?;
    let form = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::FormNotFound)),
    };

    if !form.condition.check(&project) || !form.is_visible_to_with_project(login_user, &project) {
        return Err(UseCaseError::UseCase(Error::FormNotFound));
    }

    let mut items = Vec::new();
    for item in input.items {
        let item_id = item.item_id;
        let item = match to_form_answer_item(ctx, &project, &form, item).await? {
            Ok(item) => item,
            Err(err) => return Err(UseCaseError::UseCase(Error::InvalidItem(item_id, err))),
        };
        items.push(item);
    }

    let items = form_answer::FormAnswerItems::from_items(items)
        .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;

    let created_at = DateTime::now();
    if !form.period.contains(created_at) {
        return Err(UseCaseError::UseCase(Error::OutOfAnswerPeriod));
    }

    // TODO: Enforce this in domain layer
    if ctx
        .get_form_answer_by_form_and_project(form.id, project.id)
        .await?
        .is_some()
    {
        return Err(UseCaseError::UseCase(Error::AlreadyAnswered));
    }

    form.items
        .check_answer(&items)
        .context("Failed to check form answers unexpectedly")?
        .map_err(|err| UseCaseError::UseCase(Error::from_check_error(err)))?;

    let answer = form_answer::FormAnswer {
        id: form_answer::FormAnswerId::from_uuid(Uuid::new_v4()),
        created_at,
        author_id: login_user.id.clone(),
        project_id: project.id,
        form_id: form.id,
        items,
    };
    ctx.store_form_answer(answer.clone())
        .await
        .context("Failed to store a form answer")?;
    use_case_ensure!(answer.is_visible_to_with_project(login_user, &project));
    Ok(FormAnswer::from_entity(answer))
}

async fn to_form_answer_item<C>(
    ctx: &Login<C>,
    project: &project::Project,
    form: &form::Form,
    item: InputFormAnswerItem,
) -> Result<Result<form_answer::FormAnswerItem, ItemError>, anyhow::Error>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
{
    let item_id = item.item_id.into_entity();

    let body = match item.body {
        Some(body) => body,
        None => {
            return Ok(Ok(form_answer::FormAnswerItem {
                item_id,
                body: None,
            }))
        }
    };

    let body = match body {
        InputFormAnswerItemBody::Text(answer) => {
            let answer = if let Some(answer) = answer {
                match item::FormAnswerItemText::from_string(answer) {
                    Ok(answer) => Some(answer),
                    Err(_err) => return Ok(Err(ItemError::InvalidText)),
                }
            } else {
                None
            };
            item::FormAnswerItemBody::Text(answer)
        }
        InputFormAnswerItemBody::Integer(answer) => item::FormAnswerItemBody::Integer(answer),
        InputFormAnswerItemBody::Checkbox(checks) => {
            let checks = checks.into_iter().map(CheckboxId::into_entity);
            let checks = match item::FormAnswerItemChecks::from_checked_ids(checks) {
                Ok(checks) => checks,
                Err(err) => return Ok(Err(ItemError::from_checks_error(err))),
            };
            item::FormAnswerItemBody::Checkbox(checks)
        }
        InputFormAnswerItemBody::Radio(answer) => {
            item::FormAnswerItemBody::Radio(answer.map(RadioId::into_entity))
        }
        InputFormAnswerItemBody::GridRadio(rows) => {
            let rows = rows.into_iter().map(to_row_answer);
            let rows = match item::FormAnswerItemGridRows::from_row_answers(rows) {
                Ok(rows) => rows,
                Err(err) => {
                    return Ok(Err(ItemError::from_row_answers_error(err)));
                }
            };
            item::FormAnswerItemBody::GridRadio(rows)
        }
        InputFormAnswerItemBody::File(files) => {
            let mut sharings = Vec::new();
            for file in files {
                let sharing = match to_file_sharing_answer(ctx, project, form, file).await? {
                    Ok(sharing) => sharing,
                    Err(err) => {
                        return Ok(Err(err));
                    }
                };
                sharings.push(sharing);
            }
            let sharings = match item::FormAnswerItemFileSharings::from_sharing_answers(sharings) {
                Ok(sharings) => sharings,
                Err(err) => {
                    return Ok(Err(ItemError::from_sharing_answers_error(err)));
                }
            };
            item::FormAnswerItemBody::File(sharings)
        }
    };

    Ok(Ok(form_answer::FormAnswerItem {
        item_id,
        body: Some(body),
    }))
}

async fn to_file_sharing_answer<C>(
    ctx: &Login<C>,
    project: &project::Project,
    form: &form::Form,
    file: InputFormAnswerItemFile,
) -> Result<Result<item::FileSharingAnswer, ItemError>, anyhow::Error>
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
                _ => return Ok(Err(ItemError::FileNotFound)),
            };

            let scope = file_sharing::FileSharingScope::FormAnswer(project.id, form.id);
            let sharing = match file.share_by(login_user, scope) {
                Ok(sharing) => sharing,
                Err(err) => {
                    return Ok(Err(ItemError::from_share_error(err)));
                }
            };

            ctx.store_file_sharing(sharing.clone())
                .await
                .context("Failed to store a file sharing")?;

            ensure!(sharing.scope().contains_project_form_answer(project, form));
            Ok(Ok(item::FileSharingAnswer {
                sharing_id: sharing.id(),
                type_: file.type_,
            }))
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
                _ => return Ok(Err(ItemError::FileSharingNotFound)),
            };

            if !sharing.scope().contains_project_form_answer(project, form) {
                return Ok(Err(ItemError::OutOfScopeFileSharing));
            }

            Ok(Ok(item::FileSharingAnswer {
                sharing_id: sharing.id(),
                type_: file.type_,
            }))
        }
    }
}

fn to_row_answer(answer: GridRadioRowAnswer) -> item::grid_rows::GridRadioRowAnswer {
    item::grid_rows::GridRadioRowAnswer {
        row_id: answer.row_id.into_entity(),
        value: answer.value.map(GridRadioColumnId::into_entity),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        file::FileId,
        file_sharing::FileSharingId,
        form::{
            item::{CheckboxId, FormItemId, RadioId},
            FormId,
        },
        form_answer::item::{FormAnswerItemBody, GridRadioRowAnswer},
        project::ProjectId,
    };
    use crate::{
        create_form_answer, get_project_form_answer, get_project_form_answer_shared_file,
        UseCaseError,
    };

    use sos21_domain::model::form::item;
    use sos21_domain::test;

    fn mock_input_form_answer_item_body(
        body: &item::FormItemBody,
    ) -> create_form_answer::InputFormAnswerItemBody {
        match body {
            item::FormItemBody::Text(item) => create_form_answer::InputFormAnswerItemBody::Text(
                test::model::mock_form_answer_item_text(item).map(|text| text.into_string()),
            ),
            item::FormItemBody::Integer(item) => {
                create_form_answer::InputFormAnswerItemBody::Integer(
                    test::model::mock_form_answer_item_integer(item),
                )
            }
            item::FormItemBody::Checkbox(item) => {
                create_form_answer::InputFormAnswerItemBody::Checkbox(
                    test::model::mock_form_answer_item_checkbox(item)
                        .checked_ids()
                        .map(CheckboxId::from_entity)
                        .collect(),
                )
            }
            item::FormItemBody::Radio(item) => create_form_answer::InputFormAnswerItemBody::Radio(
                test::model::mock_form_answer_item_radio(item).map(RadioId::from_entity),
            ),
            item::FormItemBody::GridRadio(item) => {
                create_form_answer::InputFormAnswerItemBody::GridRadio(
                    test::model::mock_form_answer_item_grid_radio(item)
                        .into_row_answers()
                        .map(GridRadioRowAnswer::from_entity)
                        .collect(),
                )
            }
            item::FormItemBody::File(item) => create_form_answer::InputFormAnswerItemBody::File(
                test::model::mock_form_answer_item_file(item)
                    .sharing_answers()
                    .map(|answer| {
                        create_form_answer::InputFormAnswerItemFile::Sharing(
                            FileSharingId::from_entity(answer.sharing_id),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn mock_input_form_answer_item(
        item: &item::FormItem,
    ) -> create_form_answer::InputFormAnswerItem {
        create_form_answer::InputFormAnswerItem {
            item_id: FormItemId::from_entity(item.id),
            body: Some(mock_input_form_answer_item_body(&item.body)),
        }
    }

    fn mock_input_form_answer_items(
        items: &item::FormItems,
    ) -> Vec<create_form_answer::InputFormAnswerItem> {
        items.items().map(mock_input_form_answer_item).collect()
    }

    #[tokio::test]
    async fn test_create_subowner() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project =
            test::model::new_general_project_with_subowner(owner.id.clone(), user.id.clone());
        let form = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        let project_id = ProjectId::from_entity(project.id);
        let input = create_form_answer::Input {
            form_id,
            project_id,
            items: mock_input_form_answer_items(&form.items),
        };

        let got = create_form_answer::run(&app, input).await.unwrap();
        assert!(got.form_id == form_id);
        assert!(got.project_id == project_id);

        assert!(matches!(
            get_project_form_answer::run(&app, project_id, form_id).await,
            Ok(answer)
            if answer.id == got.id
        ));
    }

    #[tokio::test]
    async fn test_create_owner() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id.clone());
        let form = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        let project_id = ProjectId::from_entity(project.id);
        let input = create_form_answer::Input {
            form_id,
            project_id,
            items: mock_input_form_answer_items(&form.items),
        };

        let got = create_form_answer::run(&app, input).await.unwrap();
        assert!(got.form_id == form_id);
        assert!(got.project_id == project_id);

        assert!(matches!(
            get_project_form_answer::run(&app, project_id, form_id).await,
            Ok(answer)
            if answer.id == got.id
        ));
    }

    #[tokio::test]
    async fn test_create_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_project(other.id.clone());
        let form = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        let project_id = ProjectId::from_entity(project.id);
        let input = create_form_answer::Input {
            form_id,
            project_id,
            items: mock_input_form_answer_items(&form.items),
        };

        assert!(matches!(
            create_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form_answer::Error::ProjectNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn test_invalid() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id.clone());
        let form = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        let project_id = ProjectId::from_entity(project.id);
        let item = mock_input_form_answer_item(form.items.items().next().unwrap());
        let input = create_form_answer::Input {
            form_id,
            project_id,
            items: vec![item.clone(), item.clone()],
        };

        assert!(matches!(
            create_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    #[tokio::test]
    async fn test_file_share() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id.clone());

        let (form, item_id) = {
            let body = item::FormItemBody::File(item::FileFormItem {
                types: None,
                accept_multiple_files: false,
                is_required: true,
            });
            let item = test::model::new_form_item_with_body(body);
            let item_id = item.id;
            let items = item::FormItems::from_items(vec![item]).unwrap();
            let form = test::model::new_form_with_items(other.id.clone(), items);
            (form, item_id)
        };
        let (file, object) = test::model::new_file(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        let answer_item = create_form_answer::InputFormAnswerItem {
            item_id: FormItemId::from_entity(item_id),
            body: Some(create_form_answer::InputFormAnswerItemBody::File(vec![
                create_form_answer::InputFormAnswerItemFile::File(FileId::from_entity(file.id)),
            ])),
        };
        let form_id = FormId::from_entity(form.id);
        let project_id = ProjectId::from_entity(project.id);
        let input = create_form_answer::Input {
            form_id,
            project_id,
            items: vec![answer_item],
        };

        let got = create_form_answer::run(&app, input).await.unwrap();
        assert_eq!(got.form_id, form_id);
        assert_eq!(got.project_id, project_id);

        let answer = get_project_form_answer::run(&app, project_id, form_id)
            .await
            .unwrap();
        assert_eq!(answer.id, got.id);
        let sharing_id = match &answer.items[0].body {
            Some(FormAnswerItemBody::File(sharings)) => sharings[0],
            _ => panic!("created form answer item is not file"),
        };

        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, get_project_form_answer_shared_file::Input {
                project_id,
                form_id,
                sharing_id
            }).await,
            Ok(got)
            if got.id == FileId::from_entity(file.id)
        ));
    }
}
