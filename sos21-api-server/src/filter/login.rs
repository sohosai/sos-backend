use crate::app::App;

use sos21_domain_context::{
    authentication::Authentication,
    login::{self, Login},
};
use warp::Rejection;

#[derive(Debug)]
pub enum LoginError {
    NotSignedUp,
    Internal(anyhow::Error),
}

impl warp::reject::Reject for LoginError {}

pub async fn login(app: Authentication<App>) -> Result<Login<App>, Rejection> {
    match Login::new(app).await {
        Ok(app) => Ok(app),
        Err(login::LoginError::NotSignedUp) => Err(warp::reject::custom(LoginError::NotSignedUp)),
        Err(login::LoginError::Internal(e)) => Err(warp::reject::custom(LoginError::Internal(e))),
    }
}
