#[derive(Debug, Clone)]
pub struct BuildLogs {
    pub content: String,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub content_hash: u64,
}
