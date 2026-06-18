use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UpstreamResolver {
    pub id: i64,
    pub ip_address: String,
    pub domain: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRule {
    pub id: i64,
    pub domain: String,
    pub client_filter: Option<String>,
    pub target_ip: String,
    pub active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<PriorityRuleRow> for PriorityRule {
    fn from(row: PriorityRuleRow) -> Self {
        Self {
            id: row.id,
            domain: row.domain,
            client_filter: row.client_filter,
            target_ip: row.target_ip,
            active: row.active != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PriorityRuleRow {
    pub id: i64,
    pub domain: String,
    pub client_filter: Option<String>,
    pub target_ip: String,
    pub active: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Statistic {
    pub id: i64,
    pub rule_id: i64,
    pub client_ip: String,
    pub requested_domain: String,
    pub original_ip: Option<String>,
    pub overridden_ip: String,
    pub timestamp: i64,
}
