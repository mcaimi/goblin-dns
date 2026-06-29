use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

// local imports
use crate::db::Database;
use crate::dns::resolver::DnsResolver;

/// In-memory cache of upstream resolvers
/// Periodically refreshes from database to stay up-to-date
pub struct ResolverCache {
    db: Arc<Database>,
    resolver: Arc<RwLock<Option<Arc<DnsResolver>>>>,
    refresh_interval: Duration,
}

impl ResolverCache {
    /// Creates a new resolver cache
    pub fn new(db: Arc<Database>, refresh_interval_secs: u64) -> Self {
        Self {
            db,
            resolver: Arc::new(RwLock::new(None)),
            refresh_interval: Duration::from_secs(refresh_interval_secs),
        }
    }

    /// Gets a reference to the current resolver
    pub async fn get_resolver(&self) -> Option<Arc<DnsResolver>> {
        let resolver = self.resolver.read().await;
        resolver.clone()
    }

    /// Manually refreshes the cache from the database
    pub async fn refresh(&self) -> Result<usize, anyhow::Error> {
        debug!("Refreshing resolver cache from database");

        let upstreams = self.db.get_all_resolvers().await?;
        let count = upstreams.len();

        if upstreams.is_empty() {
            warn!("No upstream resolvers configured in database");
            let mut resolver = self.resolver.write().await;
            *resolver = None;
            return Ok(0);
        }

        // Create a new DnsResolver from the upstream list
        match DnsResolver::from_upstreams(upstreams).await {
            Ok(new_resolver) => {
                let mut resolver = self.resolver.write().await;
                *resolver = Some(Arc::new(new_resolver));
                info!("Resolver cache refreshed - {} upstream resolvers loaded", count);
                Ok(count)
            }
            Err(e) => {
                error!("Failed to create DNS resolver from upstreams: {}", e);
                Err(e)
            }
        }
    }

    /// Starts the automatic refresh loop in the background
    /// This should be spawned as a Tokio task
    pub async fn run_refresh_loop(self: Arc<Self>) {
        info!(
            "Starting resolver cache refresh loop (interval: {:?})",
            self.refresh_interval
        );

        // Do initial refresh
        if let Err(e) = self.refresh().await {
            error!("Initial resolver cache refresh failed: {}", e);
        }

        // Periodic refresh
        let mut ticker = interval(self.refresh_interval);

        loop {
            ticker.tick().await;

            debug!("Resolver cache refresh tick");

            if let Err(e) = self.refresh().await {
                error!("Failed to refresh resolver cache: {}", e);
            }
        }
    }
}

/// Handle for accessing the resolver cache
/// This is cloneable and can be shared across tasks
#[derive(Clone)]
pub struct ResolverCacheHandle {
    cache: Arc<ResolverCache>,
}

impl ResolverCacheHandle {
    pub fn new(cache: Arc<ResolverCache>) -> Self {
        Self { cache }
    }

    /// Gets the current resolver from the cache
    pub async fn get_resolver(&self) -> Option<Arc<DnsResolver>> {
        self.cache.get_resolver().await
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
    async fn test_resolver_cache_init() {
        // This is a basic test - would need a test database for full testing
        let db_url = "sqlite::memory:";
        let db = Database::new(db_url).await.unwrap();
        let db = Arc::new(db);

        let cache = ResolverCache::new(db.clone(), 60);
        assert!(cache.get_resolver().await.is_none());
    }
}
