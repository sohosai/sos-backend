use crate::error::{UseCaseError, UseCaseResult};
use crate::model::form::{
    item::{CheckboxId, FormItemId, GridRadioColumnId, GridRadioRowId, RadioId},
    FormId,
};
use crate::model::form_answer::{
    item::{FormAnswerItemBody, GridRadioRowAnswer},
    FormAnswer, FormAnswerItem,
};
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{FormAnswerRepository, FormRepository, Login, ProjectRepository};
use sos21_domain::model::{
    date_time::DateTime,
    form,
    form_answer::{self, item},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub items: Vec<FormAnswerItem>,
}

#[derive(Debug, Clone)]
pub enum ItemError {
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
    C: ProjectRepository + FormRepository + FormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let items = input
        .items
        .into_iter()
        .map(|item| {
            let item_id = item.item_id;
            to_form_answer_item(item)
                .map_err(|err| UseCaseError::UseCase(Error::InvalidItem(item_id, err)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let items = form_answer::FormAnswerItems::from_items(items)
        .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;

    let result = ctx
        .get_project(input.project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let (project, _) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    if !project.is_visible_to(login_user) {
        return Err(UseCaseError::UseCase(Error::ProjectNotFound));
    }

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

fn to_form_answer_item(item: FormAnswerItem) -> Result<form_answer::FormAnswerItem, ItemError> {
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
        FormAnswerItemBody::Text(answer) => {
            let answer = answer
                .map(item::FormAnswerItemText::from_string)
                .transpose()
                .map_err(|_| ItemError::InvalidText)?;
            item::FormAnswerItemBody::Text(answer)
        }
        FormAnswerItemBody::Integer(answer) => item::FormAnswerItemBody::Integer(answer),
        FormAnswerItemBody::Checkbox(checks) => {
            let checks = item::FormAnswerItemChecks::from_checked_ids(
                checks.into_iter().map(CheckboxId::into_entity),
            )
            .map_err(ItemError::from_checks_error)?;
            item::FormAnswerItemBody::Checkbox(checks)
        }
        FormAnswerItemBody::Radio(answer) => {
            item::FormAnswerItemBody::Radio(answer.map(RadioId::into_entity))
        }
        FormAnswerItemBody::GridRadio(rows) => {
            let rows =
                item::FormAnswerItemGridRows::from_row_answers(rows.into_iter().map(to_row_answer))
                    .map_err(ItemError::from_row_answers_error)?;
            item::FormAnswerItemBody::GridRadio(rows)
        }
    };

    Ok(form_answer::FormAnswerItem {
        item_id,
        body: Some(body),
    })
}

fn to_row_answer(answer: GridRadioRowAnswer) -> item::grid_rows::GridRadioRowAnswer {
    item::grid_rows::GridRadioRowAnswer {
        row_id: answer.row_id.into_entity(),
        value: answer.value.map(GridRadioColumnId::into_entity),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{form::FormId, form_answer::item::FormAnswerItem, project::ProjectId};
    use crate::{create_form_answer, get_project_form_answer, UseCaseError};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_create() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
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
            items: test::model::mock_form_answer_items(&form.items)
                .into_items()
                .map(FormAnswerItem::from_entity)
                .collect(),
        };

        let result = create_form_answer::run(&app, input).await;
        assert!(result.is_ok());

        let got = result.unwrap();
        assert!(got.form_id == form_id);
        assert!(got.project_id == project_id);

        assert!(matches!(
            get_project_form_answer::run(&app, project_id, form_id).await,
            Ok(answer)
            if answer.id == got.id
        ));
    }

    #[tokio::test]
    async fn test_invalid() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
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
        let item = FormAnswerItem::from_entity(test::model::mock_form_answer_item(
            form.items.items().next().unwrap(),
        ));
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
}
