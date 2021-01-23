use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub firebase_project_id: String,
    pub jwt_keys_url: Url,
    pub postgres_uri: String,
    pub max_database_connections: u32,
}
