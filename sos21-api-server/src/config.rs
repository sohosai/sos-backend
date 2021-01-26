use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub jwt_audience: String,
    pub jwt_issuer: String,
    pub jwt_keys_url: Url,
    pub postgres_uri: String,
    pub max_database_connections: u32,
}
