use crate::app::App;
use crate::config::Config;

use anyhow::Context;
use jsonwebtoken as jwt;
use sos21_domain_context::authentication::{self, Authentication};
use tracing::{event, Level};
use warp::{Filter, Rejection};

mod bearer;
mod claim;
mod key_store;
use bearer::Bearer;
use claim::Claims;
pub use key_store::KeyStore;

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
    UnverifiedEmail,
    InvalidEmail,
    NoEmail,
    Unauthorized,
}

impl warp::reject::Reject for AuthenticationError {}

#[tracing::instrument(skip(key_store, app), level = "debug")]
async fn handle_validation(
    (key_store, app): (KeyStore, App),
    bearer: Bearer,
) -> Result<Authentication<App>, Rejection> {
    let claims = match validate_token(app.config(), key_store, bearer).await {
        Ok(cs) => cs,
        Err(error) => {
            event!(Level::INFO, %error, "Invalid token");
            return Err(warp::reject::custom(AuthenticationError::InvalidToken));
        }
    };

    let email = match claims.email {
        Some(email) => email,
        None => return Err(warp::reject::custom(AuthenticationError::NoEmail)),
    };

    if !claims.email_verified {
        return Err(warp::reject::custom(AuthenticationError::UnverifiedEmail));
    }

    match Authentication::new(app, claims.sub, email) {
        Ok(app) => Ok(app),
        Err(authentication::AuthenticationError::InvalidEmail) => {
            Err(warp::reject::custom(AuthenticationError::InvalidEmail))
        }
    }
}

pub fn authenticate(
    key_store: KeyStore,
    app: App,
) -> impl Filter<Extract = (Authentication<App>,), Error = Rejection> + Clone {
    warp::any()
        .map(move || (key_store.clone(), app.clone()))
        .and(
            warp::header::<Bearer>("authorization").or_else(|_| async {
                Err(warp::reject::custom(AuthenticationError::Unauthorized))
            }),
        )
        .and_then(handle_validation)
}
