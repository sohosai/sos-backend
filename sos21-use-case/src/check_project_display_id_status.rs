use std::convert::Infallible;

use crate::error::UseCaseResult;

use sos21_domain::{
    context::{Login, ProjectRepository},
    model::project,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIdStatus {
    Invalid { reason: DisplayIdInvalidReason },
    Unavailable,
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIdInvalidReason {
    TooLong,
    TooShort,
    ContainsDisallowedCharacter,
    StartsWithUnderscore,
}

impl DisplayIdInvalidReason {
    fn from_display_id_error(err: &project::display_id::DisplayIdError) -> Self {
        use project::display_id::DisplayIdErrorKind;

        match err.kind() {
            DisplayIdErrorKind::TooLong => DisplayIdInvalidReason::TooLong,
            DisplayIdErrorKind::TooShort => DisplayIdInvalidReason::TooShort,
            DisplayIdErrorKind::ContainsDisallowedCharacter => {
                DisplayIdInvalidReason::ContainsDisallowedCharacter
            }
            DisplayIdErrorKind::StartsWithUnderscore => {
                DisplayIdInvalidReason::StartsWithUnderscore
            }
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    display_id: String,
) -> UseCaseResult<DisplayIdStatus, Infallible>
where
    C: ProjectRepository + Send + Sync,
{
    let display_id = match project::ProjectDisplayId::from_string(display_id) {
        Err(err) => {
            return Ok(DisplayIdStatus::Invalid {
                reason: DisplayIdInvalidReason::from_display_id_error(&err),
            })
        }
        Ok(id) => id,
    };

    use_case_ensure!(display_id.is_visible_to(ctx.login_user()));
    if display_id.is_available(ctx).await? {
        Ok(DisplayIdStatus::Available)
    } else {
        Ok(DisplayIdStatus::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use crate::check_project_display_id_status::{self, DisplayIdStatus};
    use sos21_domain::test;

    // Checks that the normal user can check the display ID status.
    #[tokio::test]
    async fn test_general_other_unavailable() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            check_project_display_id_status::run(&app, project_other.display_id.into_string())
                .await,
            Ok(DisplayIdStatus::Unavailable)
        ));
    }

    // Checks that the normal user can check the (available) display ID status.
    #[tokio::test]
    async fn test_general_other_available() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            check_project_display_id_status::run(&app, project_other.display_id.into_string())
                .await,
            Ok(DisplayIdStatus::Available)
        ));
    }
}
