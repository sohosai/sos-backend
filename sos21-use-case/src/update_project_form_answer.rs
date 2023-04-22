use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::{FormId, FormItemId};
use crate::model::form_answer::FormAnswer;
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{
    FileRepository, FileSharingRepository, FormAnswerRepository, FormRepository, Login,
    ProjectRepository,
};
use sos21_domain::model::{date_time, form_answer};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub items: Vec<interface::form_answer::InputFormAnswerItem>,
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FormNotFound,
    FormAnswerNotFound,
    OutOfAnswerPeriod,
    InvalidItems(interface::form_answer::FormAnswerItemsError),
    InvalidAnswer(interface::form::CheckAnswerError),
    InsufficientPermissions,
}

impl Error {
    fn from_items_error(err: interface::form_answer::FormAnswerItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_set_error(err: form_answer::SetItemsError) -> Self {
        match err.kind() {
            form_answer::SetItemsErrorKind::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
            form_answer::SetItemsErrorKind::MismatchedItemsLength => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemsLength)
            }
            form_answer::SetItemsErrorKind::MismatchedItemId { expected, got } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemId {
                    expected: FormItemId::from_entity(expected),
                    got: FormItemId::from_entity(got),
                })
            }
            form_answer::SetItemsErrorKind::InvalidItem { id, kind } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::InvalidAnswerItem {
                    item_id: FormItemId::from_entity(id),
                    item_error: interface::form::to_check_answer_item_error(kind),
                })
            }
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
        Some(form) if form.is_visible_to_with_project(login_user, &project) => form,
        _ => return Err(UseCaseError::UseCase(Error::FormNotFound)),
    };

    let result = ctx
        .get_form_answer_by_form_and_project(form.id(), project.id())
        .await
        .context("Failed to get a form answer")?;
    let mut answer = match result {
        Some(answer) if answer.is_visible_to_with_project(login_user, &project) => answer,
        _ => return Err(UseCaseError::UseCase(Error::FormAnswerNotFound)),
    };

    // NOTE: Check the answer period before the validation for the convenience of clients
    if !form.period().contains(date_time::DateTime::now()) {
        return Err(UseCaseError::UseCase(Error::OutOfAnswerPeriod));
    }

    let items = interface::form_answer::to_form_answer_items(ctx, &project, &form, input.items)
        .await
        .map_err(|err| err.map_use_case(Error::from_items_error))?;

    answer
        .set_items(login_user, &form, &project, items)
        .map_err(|err| UseCaseError::from_domain(err, Error::from_set_error))?;

    ctx.store_form_answer(answer.clone())
        .await
        .context("Failed to store a form answer")?;
    use_case_ensure!(answer.is_visible_to_with_project(login_user, &project));
    Ok(FormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        form::{item::FormItemId, FormId},
        form_answer::item::FormAnswerItemBody,
        project::ProjectId,
    };
    use crate::{interface, update_project_form_answer, UseCaseError};

    use sos21_domain::model::form::item;
    use sos21_domain::model::form_answer::item as answer_item;
    use sos21_domain::test;

    fn prepare_items() -> (
        item::FormItemId,
        item::FormItems,
        answer_item::FormAnswerItems,
    ) {
        let body = item::FormItemBody::Integer(
            item::IntegerFormItem::from_content(item::integer::IntegerFormItemContent {
                is_required: false,
                max: None,
                min: None,
                placeholder: None,
                unit: None,
            })
            .unwrap(),
        );
        let item = test::model::new_form_item_with_body(body);
        let item_id = item.id;
        let items = item::FormItems::from_items(vec![item]).unwrap();
        let answer_item = answer_item::FormAnswerItem {
            item_id,
            body: Some(answer_item::FormAnswerItemBody::Integer(Some(10))),
        };
        let answer_items = answer_item::FormAnswerItems::from_items(vec![answer_item]).unwrap();
        (item_id, items, answer_items)
    }

    #[tokio::test]
    async fn test_answer_in_period_general_owner() {
        let owner = test::model::new_general_user();
        let project = test::model::new_general_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (item_id, items, answer_items) = prepare_items();

        let item_id = FormItemId::from_entity(item_id);
        let form = test::model::new_form_with_items(operator.id().clone(), items);
        let form_answer = test::model::new_form_answer_with_items(
            owner.id().clone(),
            &project,
            &form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![form_answer.clone()])
            .build()
            .login_as(owner)
            .await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id,
            body: Some(interface::form_answer::InputFormAnswerItemBody::Integer(
                Some(20),
            )),
        };
        let input = update_project_form_answer::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            items: vec![answer_item],
        };
        let answer = update_project_form_answer::run(&app, input).await.unwrap();

        assert_eq!(answer.items[0].item_id, item_id);
        assert_eq!(
            answer.items[0].body,
            Some(FormAnswerItemBody::Integer(Some(20)))
        );
    }

    #[tokio::test]
    async fn test_answer_in_period_general_subowner() {
        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();
        let project = test::model::new_general_project_with_subowner(
            owner.id().clone(),
            subowner.id().clone(),
        );
        let operator = test::model::new_operator_user();

        let (item_id, items, answer_items) = prepare_items();

        let item_id = FormItemId::from_entity(item_id);
        let form = test::model::new_form_with_items(operator.id().clone(), items);
        let form_answer = test::model::new_form_answer_with_items(
            owner.id().clone(),
            &project,
            &form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone(), subowner.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![form_answer.clone()])
            .build()
            .login_as(subowner)
            .await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id,
            body: Some(interface::form_answer::InputFormAnswerItemBody::Integer(
                Some(20),
            )),
        };
        let input = update_project_form_answer::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            items: vec![answer_item],
        };
        let answer = update_project_form_answer::run(&app, input).await.unwrap();

        assert_eq!(answer.items[0].item_id, item_id);
        assert_eq!(
            answer.items[0].body,
            Some(FormAnswerItemBody::Integer(Some(20)))
        );
    }

    #[tokio::test]
    async fn test_answer_in_period_general_other() {
        let owner = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project = test::model::new_general_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (item_id, items, answer_items) = prepare_items();

        let item_id = FormItemId::from_entity(item_id);
        let form = test::model::new_form_with_items(operator.id().clone(), items);
        let form_answer = test::model::new_form_answer_with_items(
            owner.id().clone(),
            &project,
            &form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![form_answer.clone()])
            .build()
            .login_as(other)
            .await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id,
            body: Some(interface::form_answer::InputFormAnswerItemBody::Integer(
                Some(20),
            )),
        };
        let input = update_project_form_answer::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            items: vec![answer_item],
        };
        assert!(matches!(
            update_project_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project_form_answer::Error::ProjectNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn test_answer_after_period_owner() {
        let owner = test::model::new_general_user();
        let project = test::model::new_general_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let period = test::model::new_form_period_to_now();
        let form = test::model::new_form_with_period(operator.id().clone(), period);
        let form_answer = test::model::new_form_answer(owner.id().clone(), &project, &form);

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![form_answer.clone()])
            .build()
            .login_as(owner)
            .await;

        let input = update_project_form_answer::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            items: vec![],
        };
        assert!(matches!(
            update_project_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project_form_answer::Error::OutOfAnswerPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_answer_in_period_invalid() {
        let owner = test::model::new_general_user();
        let project = test::model::new_general_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let form = test::model::new_form(operator.id().clone());
        let form_answer = test::model::new_form_answer(owner.id().clone(), &project, &form);

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![form_answer.clone()])
            .build()
            .login_as(owner)
            .await;

        let input = update_project_form_answer::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            items: vec![],
        };
        assert!(matches!(
            update_project_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project_form_answer::Error::InvalidItems(..)
            ))
        ));
    }
}
