use anyhow::Result;
use sos21_model::user::{User, UserId};

#[async_trait::async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: User) -> Result<()>;
    async fn get_user(&self, id: UserId) -> Result<Option<User>>;
}
