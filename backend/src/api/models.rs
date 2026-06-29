use serde::{Deserialize, Serialize};

// ========== Upstream Resolver API Models ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResolverRequest {
    pub ip_address: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResolverRequest {
    pub ip_address: Option<String>,
    pub domain: Option<String>,
}

// ========== Priority Rule API Models ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub domain: String,
    pub client_filter: Option<String>,
    pub target_ip: String,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    pub domain: Option<String>,
    pub client_filter: Option<Option<String>>,
    pub target_ip: Option<String>,
    pub active: Option<bool>,
}

// ========== Statistics API Models ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Serialize)]
pub struct StatsSummaryResponse {
    pub total_hits: i64,
    pub unique_clients: i64,
    pub active_rules: i64,
}

// ========== Generic API Responses ==========

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub id: i64,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub deleted: bool,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub updated: bool,
}
