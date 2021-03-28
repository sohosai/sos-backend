use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::file_sharing::FileSharing;

use anyhow::Context;
use sos21_domain::context::{FileSharingRepository, Login};

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<FileSharing>, Infallible>
where
    C: FileSharingRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let sharings = ctx
        .list_file_sharings_by_user(login_user.id.clone())
        .await
        .context("Failed to list file sharings")?;

    use_case_ensure!(sharings.iter().all(|(sharing, file)| sharing
        .is_visible_to_with_file(login_user, &file)
        && file.is_visible_to(login_user)));

    let sharings = sharings
        .into_iter()
        .map(|(sharing, file)| FileSharing::from_entity(sharing, file))
        .collect();
    Ok(sharings)
}

#[cfg(test)]
mod tests {
    use crate::list_user_file_sharings;
    use crate::model::file_sharing::FileSharingId;

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        use std::collections::HashSet;

        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (file1, object1) = test::model::new_file(user.id.clone());
        let (file2, object2) = test::model::new_file(user.id.clone());
        let sharing1 =
            file_sharing::FileSharing::new(file1.id, file_sharing::FileSharingScope::Public);
        let sharing2 =
            file_sharing::FileSharing::new(file2.id, file_sharing::FileSharingScope::Committee);
        let sharing3 = file_sharing::FileSharing::new(
            file2.id,
            file_sharing::FileSharingScope::CommitteeOperator,
        );
        let (file3, object3) = test::model::new_file(other.id.clone());
        let sharing4 =
            file_sharing::FileSharing::new(file3.id, file_sharing::FileSharingScope::Public);
        let sharing5 =
            file_sharing::FileSharing::new(file3.id, file_sharing::FileSharingScope::Committee);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .files(vec![file1.clone(), file2.clone(), file3.clone()])
            .objects(vec![object1, object2, object3])
            .await
            .sharings(vec![
                sharing1.clone(),
                sharing2.clone(),
                sharing3.clone(),
                sharing4.clone(),
                sharing5.clone(),
            ])
            .build()
            .login_as(user)
            .await;

        let sharings = list_user_file_sharings::run(&app).await.unwrap();

        let got: HashSet<_> = sharings.into_iter().map(|sharing| sharing.id).collect();
        let expected: HashSet<_> = (&[
            FileSharingId::from_entity(sharing1.id()),
            FileSharingId::from_entity(sharing2.id()),
            FileSharingId::from_entity(sharing3.id()),
        ])
            .iter()
            .cloned()
            .collect();
        assert_eq!(got, expected);
    }
}
