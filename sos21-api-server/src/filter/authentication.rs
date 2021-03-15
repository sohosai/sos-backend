use crate::config::Config;

use anyhow::Context as _;
use jsonwebtoken as jwt;
use tracing::{event, Level};
use warp::{Filter, Rejection};

mod bearer;
mod claim;
mod key_store;
use bearer::Bearer;
use claim::Claims;
pub use key_store::KeyStore;

#[derive(Debug, Clone)]
pub struct AuthenticationInfo {
    pub user_id: String,
    pub email: String,
}

#[tracing::instrument(skip(config, key_store), level = "debug")]
async fn validate_token(
    config: &Config,
    key_store: KeyStore,
    bearer: Bearer,
) -> Result<Claims, anyhow::Error> {
    let header = jwt::decode_header(&bearer.token)?;
    let kid = header.kid.context("No key ID found in JWT header")?;
    let key = key_store.get(&kid).await.context("Unknown key ID")?;
    let validation = jwt::Validation {
        leeway: 0,
        validate_exp: true,
        validate_nbf: false,
        aud: Some(std::iter::once(config.jwt_audience.clone()).collect()),
        iss: Some(config.jwt_issuer.clone()),
        sub: None,
        algorithms: vec![jwt::Algorithm::RS256],
    };
    let data = jwt::decode(&bearer.token, &key, &validation).context("Failed to validate JWT")?;
    Ok(data.claims)
}

#[derive(Debug)]
pub enum AuthenticationError {
    InvalidToken,
    UnverifiedEmailAddress,
    NoEmailAddress,
    Unauthorized,
}

impl warp::reject::Reject for AuthenticationError {}

#[tracing::instrument(skip(key_store, config), level = "debug")]
async fn handle_validation(
    key_store: KeyStore,
    config: Config,
    bearer: Bearer,
) -> Result<AuthenticationInfo, Rejection> {
    let claims = match validate_token(&config, key_store, bearer).await {
        Ok(cs) => cs,
        Err(error) => {
            event!(Level::INFO, ?error, "Invalid token");
            return Err(warp::reject::custom(AuthenticationError::InvalidToken));
        }
    };

    let email = match claims.email {
        Some(email) => email,
        None => return Err(warp::reject::custom(AuthenticationError::NoEmailAddress)),
    };

    if !claims.email_verified {
        return Err(warp::reject::custom(
            AuthenticationError::UnverifiedEmailAddress,
        ));
    }

    Ok(AuthenticationInfo {
        user_id: claims.sub,
        email,
    })
}

pub fn authenticate(
    key_store: KeyStore,
    config: Config,
) -> impl Filter<Extract = (AuthenticationInfo,), Error = Rejection> + Clone {
    warp::any()
        .map(move || key_store.clone())
        .and(warp::any().map(move || config.clone()))
        .and(
            warp::header::<Bearer>("authorization").or_else(|_| async {
                Err(warp::reject::custom(AuthenticationError::Unauthorized))
            }),
        )
        .and_then(handle_validation)
}
