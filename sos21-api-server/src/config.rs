use chrono::{DateTime, Utc};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub jwt_audience: String,
    pub jwt_issuer: String,
    pub jwt_keys_url: Url,
    pub postgres_uri: String,
    pub max_database_connections: u32,
    pub s3_access_key: String,
    pub s3_access_secret: String,
    pub s3_region: String,
    pub s3_endpoint: String,
    pub s3_object_bucket: String,
    pub administrator_email: String,
    pub start_project_creation_period: Option<DateTime<Utc>>,
    pub end_project_creation_period: Option<DateTime<Utc>>,
}
