use sqlx::SqlitePool;
use crate::db::models::{UpstreamResolver, PriorityRule, PriorityRuleRow, Statistic};
use anyhow::Result;
use serde::Serialize;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Enable WAL mode for better concurrent access
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    // Upstream Resolvers
    pub async fn get_all_resolvers(&self) -> Result<Vec<UpstreamResolver>> {
        let resolvers = sqlx::query_as::<_, UpstreamResolver>(
            "SELECT * FROM upstream_resolvers ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(resolvers)
    }

    pub async fn get_resolver_by_id(&self, id: i64) -> Result<Option<UpstreamResolver>> {
        let resolver = sqlx::query_as::<_, UpstreamResolver>(
            "SELECT * FROM upstream_resolvers WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(resolver)
    }

    pub async fn create_resolver(&self, ip_address: &str, domain: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            "INSERT INTO upstream_resolvers (ip_address, domain, created_at, updated_at) VALUES (?, ?, ?, ?)"
        )
        .bind(ip_address)
        .bind(domain)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn update_resolver(&self, id: i64, ip_address: Option<&str>, domain: Option<&str>) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();

        let existing = self.get_resolver_by_id(id).await?;
        if existing.is_none() {
            return Ok(false);
        }
        let existing = existing.unwrap();

        let new_ip = ip_address.unwrap_or(&existing.ip_address);
        let new_domain = domain.unwrap_or(&existing.domain);

        sqlx::query(
            "UPDATE upstream_resolvers SET ip_address = ?, domain = ?, updated_at = ? WHERE id = ?"
        )
        .bind(new_ip)
        .bind(new_domain)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn delete_resolver(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM upstream_resolvers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Priority Rules
    pub async fn get_all_rules(&self) -> Result<Vec<PriorityRule>> {
        let rows = sqlx::query_as::<_, PriorityRuleRow>(
            "SELECT * FROM priority_rules ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_active_rules(&self) -> Result<Vec<PriorityRule>> {
        let rows = sqlx::query_as::<_, PriorityRuleRow>(
            "SELECT * FROM priority_rules WHERE active = 1 ORDER BY domain"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_rule_by_id(&self, id: i64) -> Result<Option<PriorityRule>> {
        let row = sqlx::query_as::<_, PriorityRuleRow>(
            "SELECT * FROM priority_rules WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn create_rule(&self, domain: &str, client_filter: Option<&str>, target_ip: &str, active: bool) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let active_int = if active { 1 } else { 0 };

        let result = sqlx::query(
            "INSERT INTO priority_rules (domain, client_filter, target_ip, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(domain)
        .bind(client_filter)
        .bind(target_ip)
        .bind(active_int)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn update_rule(&self, id: i64, domain: Option<&str>, client_filter: Option<Option<&str>>, target_ip: Option<&str>, active: Option<bool>) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();

        let existing = self.get_rule_by_id(id).await?;
        if existing.is_none() {
            return Ok(false);
        }
        let existing = existing.unwrap();

        let new_domain = domain.unwrap_or(&existing.domain);
        let new_client_filter = client_filter.unwrap_or(existing.client_filter.as_deref());
        let new_target_ip = target_ip.unwrap_or(&existing.target_ip);
        let new_active = active.unwrap_or(existing.active);
        let active_int = if new_active { 1 } else { 0 };

        sqlx::query(
            "UPDATE priority_rules SET domain = ?, client_filter = ?, target_ip = ?, active = ?, updated_at = ? WHERE id = ?"
        )
        .bind(new_domain)
        .bind(new_client_filter)
        .bind(new_target_ip)
        .bind(active_int)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn toggle_rule(&self, id: i64) -> Result<bool> {
        let existing = self.get_rule_by_id(id).await?;
        if existing.is_none() {
            return Ok(false);
        }
        let existing = existing.unwrap();

        let now = chrono::Utc::now().timestamp();
        let new_active = if existing.active { 0 } else { 1 };

        sqlx::query("UPDATE priority_rules SET active = ?, updated_at = ? WHERE id = ?")
            .bind(new_active)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(true)
    }

    pub async fn delete_rule(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM priority_rules WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Statistics
    pub async fn record_stat(&self, rule_id: i64, client_ip: &str, requested_domain: &str, original_ip: Option<&str>, overridden_ip: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO statistics (rule_id, client_ip, requested_domain, original_ip, overridden_ip, timestamp) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(rule_id)
        .bind(client_ip)
        .bind(requested_domain)
        .bind(original_ip)
        .bind(overridden_ip)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_stats(&self, limit: i64, offset: i64) -> Result<Vec<Statistic>> {
        let stats = sqlx::query_as::<_, Statistic>(
            "SELECT * FROM statistics ORDER BY timestamp DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_stats_for_rule(&self, rule_id: i64) -> Result<Vec<Statistic>> {
        let stats = sqlx::query_as::<_, Statistic>(
            "SELECT * FROM statistics WHERE rule_id = ? ORDER BY timestamp DESC"
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_stats_summary(&self) -> Result<StatsSummary> {
        let total_hits: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM statistics")
            .fetch_one(&self.pool)
            .await?;

        let unique_clients: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT client_ip) FROM statistics")
            .fetch_one(&self.pool)
            .await?;

        let active_rules: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM priority_rules WHERE active = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(StatsSummary {
            total_hits: total_hits.0,
            unique_clients: unique_clients.0,
            active_rules: active_rules.0,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct StatsSummary {
    pub total_hits: i64,
    pub unique_clients: i64,
    pub active_rules: i64,
}
