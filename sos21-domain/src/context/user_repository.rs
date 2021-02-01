use crate::model::user::{User, UserId};

use anyhow::Result;

#[async_trait::async_trait]
pub trait UserRepository {
    async fn store_user(&self, user: User) -> Result<()>;
    async fn get_user(&self, id: UserId) -> Result<Option<User>>;
    async fn list_users(&self) -> Result<Vec<User>>;
}

#[macro_export]
macro_rules! delegate_user_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? UserRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::UserRepository for $ty {
            async fn store_user(
                &$sel,
                user: $crate::model::user::User,
            ) -> ::anyhow::Result<()> {
                $target.store_user(user).await
            }
            async fn get_user(
                &$sel,
                id: $crate::model::user::UserId,
            ) -> ::anyhow::Result<Option<$crate::model::user::User>> {
                $target.get_user(id).await
            }
            async fn list_users(
                &$sel,
            ) -> ::anyhow::Result<Vec<$crate::model::user::User>> {
                $target.list_users().await
            }
        }
    };
}

#[async_trait::async_trait]
impl<C: UserRepository + Sync> UserRepository for &C {
    async fn store_user(&self, user: User) -> Result<()> {
        <C as UserRepository>::store_user(self, user).await
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        <C as UserRepository>::get_user(self, id).await
    }

    async fn list_users(&self) -> Result<Vec<User>> {
        <C as UserRepository>::list_users(self).await
    }
}
