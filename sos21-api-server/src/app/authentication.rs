use anyhow::Result;
use chrono::{DateTime, Utc};
use sos21_context::{AuthenticationContext, TimeContext, UserRepository};
use sos21_model::{
    email::EmailAddress,
    user::{User, UserId},
};

pub struct Authentication<C> {
    ctx: C,
    user_id: UserId,
    email: EmailAddress,
}

impl<C> Authentication<C> {
    pub fn new(ctx: C, user_id: UserId, email: EmailAddress) -> Self {
        Authentication {
            ctx,
            user_id,
            email,
        }
    }
}

impl<C> AuthenticationContext for Authentication<C> {
    fn login_user(&self) -> UserId {
        self.user_id.clone()
    }

    fn login_email(&self) -> EmailAddress {
        self.email.clone()
    }
}

#[async_trait::async_trait]
impl<C: UserRepository + Send + Sync> UserRepository for Authentication<C> {
    async fn create_user(&self, user: User) -> Result<()> {
        self.ctx.create_user(user).await
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        self.ctx.get_user(id).await
    }
}

impl<C: TimeContext> TimeContext for Authentication<C> {
    fn now(&self) -> DateTime<Utc> {
        self.ctx.now()
    }
}
