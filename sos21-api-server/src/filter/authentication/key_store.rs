use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use futures::future::TryFutureExt;
use jsonwebtoken::DecodingKey;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{event, span, Instrument, Level};
use url::Url;

#[derive(Debug, Clone)]
pub struct KeyStore {
    url: Url,
    client: Client,
    keys: Arc<RwLock<HashMap<String, DecodingKey<'static>>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct Key {
    n: String,
    kid: String,
    e: String,
    // ignored since they're not used
    // #[serde(rename = "use")]
    // use_: String,
    // alg: String,
    // kty: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Response {
    keys: Vec<Key>,
}

impl KeyStore {
    pub fn new(url: Url) -> Self {
        KeyStore {
            url,
            client: Client::new(),
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh(&self) -> Result<Option<u64>> {
        let response = self
            .client
            .get(self.url.clone())
            .send()
            .await
            .context("Failed to fecth from JWT keys URL")?;

        let status = response.status();
        if !status.is_success() {
            response
                .text()
                .map_ok(|text| event!(Level::ERROR, response = %text, %status))
                .instrument(span!(
                    Level::DEBUG,
                    "JWT keys URL returned non-successful status code",
                    %status
                ))
                .await?;
            bail!("JWT keys URL returned non-successful status code");
        }
        let max_age = response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(parse_max_age);

        let response: Response = response
            .json()
            .await
            .context("Failed to obtain keys from the response")?;
        let keys = response
            .keys
            .into_iter()
            .map(|key| {
                (
                    key.kid,
                    DecodingKey::from_rsa_components(&key.n, &key.e).into_static(),
                )
            })
            .collect();
        *self.keys.write().await = keys;

        Ok(max_age)
    }

    pub async fn get<T>(&self, kid: &T) -> Option<DecodingKey<'static>>
    where
        T: Hash + Eq,
        String: Borrow<T>,
    {
        self.keys.read().await.get(kid).cloned()
    }
}

// https://tools.ietf.org/html/rfc2616#section-14.9
fn parse_max_age(v: &reqwest::header::HeaderValue) -> Option<u64> {
    let s = v.to_str().ok()?.to_ascii_lowercase();
    let mut sp = s.splitn(2, "max-age");
    sp.next()?;
    let after_max_age = sp.next()?;
    debug_assert!(sp.next().is_none());
    let after_equal = after_max_age.trim().strip_prefix('=')?;
    let mut sp = after_equal
        .trim()
        .splitn(2, |c| matches!(c, ',' | '\n' | '\r' | ' ' | '\t'));
    let delta_seconds = sp.next()?;
    delta_seconds.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_max_age;
    use reqwest::header::HeaderValue;

    #[test]
    fn test_parse_max_age() {
        assert_eq!(
            parse_max_age(&HeaderValue::from_static("max-age=5000")),
            Some(5000)
        );
        assert_eq!(
            parse_max_age(&HeaderValue::from_static("no-store, max-age=0")),
            Some(0)
        );
        assert_eq!(
            parse_max_age(&HeaderValue::from_static(
                "public, max-age=604800, immutable"
            )),
            Some(604800)
        );
        assert_eq!(
            parse_max_age(&HeaderValue::from_static("  MAX-AGE =  60   , no-store")),
            Some(60)
        );
        assert_eq!(
            parse_max_age(&HeaderValue::from_static("max-age=\"60\"")),
            None
        );
    }
}
