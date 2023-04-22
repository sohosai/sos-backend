use crate::model::project_creation_availability::ProjectCreationAvailability;

use chrono::Utc;
use sos21_domain::context::ConfigContext;
use sos21_domain::model::date_time::DateTime;
use sos21_domain::model::project::ProjectCategory;

#[tracing::instrument(skip(ctx))]
pub fn run<C>(ctx: C) -> ProjectCreationAvailability
where
    C: ConfigContext + Send + Sync,
{
    let now = Utc::now();
    let now_entity = DateTime::from_utc(now);

    ProjectCreationAvailability {
        timestamp: now,
        general: ctx
            .project_creation_period_for(ProjectCategory::General)
            .contains(now_entity),
        cooking_requiring_preparation_area: ctx
            .project_creation_period_for(ProjectCategory::CookingRequiringPreparationArea)
            .contains(now_entity),
        cooking: ctx
            .project_creation_period_for(ProjectCategory::Cooking)
            .contains(now_entity),
        food: ctx
            .project_creation_period_for(ProjectCategory::Food)
            .contains(now_entity),
        stage: ctx
            .project_creation_period_for(ProjectCategory::Stage)
            .contains(now_entity),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sos21_domain::model::date_time::DateTime;
    use sos21_domain::model::project::ProjectCategory;
    use sos21_domain::model::project_creation_period::ProjectCreationPeriod;

    use crate::get_project_creation_availability;
    use crate::model::project_creation_availability::ProjectCreationAvailability;
    use sos21_domain::test;

    // Checks that it returns correct project creation availability at runtime
    #[tokio::test]
    async fn test_availability() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let now = Utc::now();
        let past_period = ProjectCreationPeriod::from_datetime(
            DateTime::from_utc(now - Duration::minutes(10)),
            DateTime::from_utc(now - Duration::minutes(5)),
        )
        .unwrap();
        let ongoing_period = ProjectCreationPeriod::from_datetime(
            DateTime::from_utc(now - Duration::minutes(5)),
            DateTime::from_utc(now + Duration::minutes(5)),
        )
        .unwrap();
        let future_period = ProjectCreationPeriod::from_datetime(
            DateTime::from_utc(now + Duration::minutes(5)),
            DateTime::from_utc(now + Duration::minutes(10)),
        )
        .unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .project_creation_period_for(
                ProjectCategory::General,
                ProjectCreationPeriod::never(),
            )
            .project_creation_period_for(ProjectCategory::CookingRequiringPreparationArea, future_period)
            .project_creation_period_for(ProjectCategory::Cooking, ongoing_period)
            .project_creation_period_for(ProjectCategory::Food, ongoing_period)
            .project_creation_period_for(ProjectCategory::Stage, past_period)
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_creation_availability::run(&app),
            ProjectCreationAvailability {
                timestamp: _,
                general: false,
                cooking_requiring_preparation_area: false,
                cooking: true,
                food: true,
                stage: false
            }
        ));
    }
}
