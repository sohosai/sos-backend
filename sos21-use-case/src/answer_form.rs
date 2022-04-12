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
use sos21_domain::model::{date_time::DateTime, form};

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
    OutOfAnswerPeriod,
    AlreadyAnswered,
    InvalidItems(interface::form_answer::FormAnswerItemsError),
    InvalidAnswer(interface::form::CheckAnswerError),
}

impl Error {
    fn from_items_error(err: interface::form_answer::FormAnswerItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_answer_error(err: form::AnswerError) -> Self {
        match err.kind() {
            form::AnswerErrorKind::OutOfAnswerPeriod => Error::OutOfAnswerPeriod,
            form::AnswerErrorKind::AlreadyAnswered => Error::AlreadyAnswered,
            form::AnswerErrorKind::NotTargeted => Error::FormNotFound,
            form::AnswerErrorKind::MismatchedItemsLength => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemsLength)
            }
            form::AnswerErrorKind::MismatchedItemId { expected, got } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemId {
                    expected: FormItemId::from_entity(expected),
                    got: FormItemId::from_entity(got),
                })
            }
            form::AnswerErrorKind::InvalidItem { id, kind } => {
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

    // NOTE: Check the answer period before the validation for the convenience of clients
    if !form.period().contains(DateTime::now()) {
        return Err(UseCaseError::UseCase(Error::OutOfAnswerPeriod));
    }

    let items = interface::form_answer::to_form_answer_items(ctx, &project, &form, input.items)
        .await
        .map_err(|err| err.map_use_case(Error::from_items_error))?;

    let answer = form
        .answer_by(ctx, login_user, &project, items)
        .await
        .map_err(|err| UseCaseError::from_domain(err, Error::from_answer_error))?;
    ctx.store_form_answer(answer.clone())
        .await
        .context("Failed to store a form answer")?;
    use_case_ensure!(answer.is_visible_to_with_project(login_user, &project));
    Ok(FormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        file::FileId,
        form::{item::FormItemId, FormId},
        form_answer::item::FormAnswerItemBody,
        project::ProjectId,
    };
    use crate::test::interface as test_interface;
    use crate::{
        answer_form, get_project_form_answer, get_project_form_answer_shared_file, interface,
        UseCaseError,
    };

    use sos21_domain::model::form::item;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_create_subowner() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_online_project_with_subowner(
            owner.id().clone(),
            user.id().clone(),
        );
        let form = test::model::new_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id());
        let project_id = ProjectId::from_entity(project.id());
        let input = answer_form::Input {
            form_id,
            project_id,
            items: test_interface::mock_input_form_answer_items(form.items()),
        };

        let got = answer_form::run(&app, input).await.unwrap();
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
        let project = test::model::new_general_online_project(user.id().clone());
        let form = test::model::new_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id());
        let project_id = ProjectId::from_entity(project.id());
        let input = answer_form::Input {
            form_id,
            project_id,
            items: test_interface::mock_input_form_answer_items(form.items()),
        };

        let got = answer_form::run(&app, input).await.unwrap();
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
        let project = test::model::new_general_online_project(other.id().clone());
        let form = test::model::new_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id());
        let project_id = ProjectId::from_entity(project.id());
        let input = answer_form::Input {
            form_id,
            project_id,
            items: test_interface::mock_input_form_answer_items(form.items()),
        };

        assert!(matches!(
            answer_form::run(&app, input).await,
            Err(UseCaseError::UseCase(answer_form::Error::ProjectNotFound))
        ));
    }

    #[tokio::test]
    async fn test_invalid() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_online_project(user.id().clone());
        let form = test::model::new_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id());
        let project_id = ProjectId::from_entity(project.id());
        let item =
            test_interface::mock_input_form_answer_item(form.items().items().next().unwrap());
        let input = answer_form::Input {
            form_id,
            project_id,
            items: vec![item.clone(), item.clone()],
        };

        assert!(matches!(
            answer_form::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    #[tokio::test]
    async fn test_file_share() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let project = test::model::new_general_online_project(user.id().clone());

        let (form, item_id) = {
            let body = item::FormItemBody::File(item::FileFormItem {
                types: None,
                accept_multiple_files: false,
                is_required: true,
            });
            let item = test::model::new_form_item_with_body(body);
            let item_id = item.id;
            let items = item::FormItems::from_items(vec![item]).unwrap();
            let form = test::model::new_form_with_items(other.id().clone(), items);
            (form, item_id)
        };
        let (file, object) = test::model::new_file(user.id().clone());

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

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id: FormItemId::from_entity(item_id),
            body: Some(interface::form_answer::InputFormAnswerItemBody::File(vec![
                interface::form_answer::InputFormAnswerItemFile::File(FileId::from_entity(file.id)),
            ])),
        };
        let form_id = FormId::from_entity(form.id());
        let project_id = ProjectId::from_entity(project.id());
        let input = answer_form::Input {
            form_id,
            project_id,
            items: vec![answer_item],
        };

        let got = answer_form::run(&app, input).await.unwrap();
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
