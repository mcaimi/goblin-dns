use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Resolver {
    pub id: u64,
    pub ip_address: String,
    pub domain: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Rule {
    pub id: u64,
    pub domain: String,
    pub client_filter: Option<String>,
    pub target_ip: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct StatEntry {
    pub id: u64,
    pub rule_id: u64,
    pub client_ip: String,
    pub requested_domain: String,
    pub original_ip: Option<String>,
    pub overridden_ip: String,
    pub timestamp: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct StatsSummary {
    pub total_hits: u64,
    pub unique_clients: u64,
    pub active_rules: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug)]
pub enum GoblinError {
    HttpError(String),
}

impl std::fmt::Display for GoblinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoblinError::HttpError(msg) => write!(f, "HTTP Error: {}", msg),
        }
    }
}

impl std::error::Error for GoblinError {}
