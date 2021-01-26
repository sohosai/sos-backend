use anyhow::Result;
use sos21_domain_model::user::{User, UserId};

#[async_trait::async_trait]
pub trait UserRepository {
    async fn store_user(&self, user: User) -> Result<()>;
    async fn get_user(&self, id: UserId) -> Result<Option<User>>;
    async fn list_users(&self) -> Result<Vec<User>>;
}

#[macro_export]
macro_rules! delegate_user_repository {
    (impl <$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*> for $t:ty : $field:ident) => {
        #[::async_trait::async_trait]
        impl<$($vars$(: $c0 $(+ $cs)* )?,)*> $crate::UserRepository for $t {
            async fn store_user(
                &self,
                user: ::sos21_domain_model::user::User,
            ) -> ::anyhow::Result<()> {
                self.$field.store_user(user).await
            }
            async fn get_user(
                &self,
                id: ::sos21_domain_model::user::UserId,
            ) -> ::anyhow::Result<Option<::sos21_domain_model::user::User>> {
                self.$field.get_user(id).await
            }
            async fn list_users(
                &self,
            ) -> ::anyhow::Result<Vec<::sos21_domain_model::user::User>> {
                self.$field.list_users().await
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
