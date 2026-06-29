use crate::db::{Database, PriorityRule};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

/// In-memory cache of priority rules
/// Periodically refreshes from database to stay up-to-date
pub struct RuleCache {
    db: Arc<Database>,
    rules: Arc<RwLock<Vec<PriorityRule>>>,
    refresh_interval: Duration,
}

impl RuleCache {
    /// Creates a new rule cache
    pub fn new(db: Arc<Database>, refresh_interval_secs: u64) -> Self {
        Self {
            db,
            rules: Arc::new(RwLock::new(Vec::new())),
            refresh_interval: Duration::from_secs(refresh_interval_secs),
        }
    }

    /// Gets a clone of the current rules (read-only)
    pub async fn get_rules(&self) -> Vec<PriorityRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// Gets the number of cached rules
    pub async fn count(&self) -> usize {
        let rules = self.rules.read().await;
        rules.len()
    }

    /// Manually refreshes the cache from the database
    pub async fn refresh(&self) -> Result<usize, anyhow::Error> {
        debug!("Refreshing rule cache from database");

        let active_rules = self.db.get_active_rules().await?;
        let total_count = self.count().await;
        let active_count = active_rules.len();

        let mut rules = self.rules.write().await;
        *rules = active_rules;

        info!("Rule cache refreshed - {}/{} active rules loaded", active_count, total_count);
        Ok(active_count)
    }

    /// Starts the automatic refresh loop in the background
    /// This should be spawned as a Tokio task
    pub async fn run_refresh_loop(self: Arc<Self>) {
        info!(
            "Starting rule cache refresh loop (interval: {:?})",
            self.refresh_interval
        );

        // Do initial refresh
        if let Err(e) = self.refresh().await {
            error!("Initial rule cache refresh failed: {}", e);
        }

        // Periodic refresh
        let mut ticker = interval(self.refresh_interval);

        loop {
            ticker.tick().await;

            debug!("Rule cache refresh tick");

            if let Err(e) = self.refresh().await {
                error!("Failed to refresh rule cache: {}", e);
            }
        }
    }
}

/// Handle for accessing the rule cache
/// This is cloneable and can be shared across tasks
#[derive(Clone)]
pub struct RuleCacheHandle {
    cache: Arc<RuleCache>,
}

impl RuleCacheHandle {
    pub fn new(cache: Arc<RuleCache>) -> Self {
        Self { cache }
    }

    /// Gets the current rules from the cache
    pub async fn get_rules(&self) -> Vec<PriorityRule> {
        self.cache.get_rules().await
    }

    /// Gets the number of cached rules
    pub async fn count(&self) -> usize {
        self.cache.count().await
    }

    /// Manually triggers a cache refresh
    pub async fn refresh(&self) -> Result<usize, anyhow::Error> {
        self.cache.refresh().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[tokio::test]
    async fn test_rule_cache_init() {
        // This is a basic test - would need a test database for full testing
        let db_url = "sqlite::memory:";
        let db = Database::new(db_url).await.unwrap();
        let db = Arc::new(db);

        let cache = RuleCache::new(db.clone(), 60);
        assert_eq!(cache.count().await, 0);
    }
}
