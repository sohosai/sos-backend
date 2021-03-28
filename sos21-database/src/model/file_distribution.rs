use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileDistribution {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FileDistributionFile {
    pub project_id: Uuid,
    pub sharing_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct FileDistributionData {
    pub distribution: FileDistribution,
    pub files: Vec<FileDistributionFile>,
}
