mod config;
mod db;
mod dns;
mod rules;
mod stats;
mod api;

use config::Config;
use db::Database;
use dns::{DnsResolver, DnsServer, DnsHandler};
use rules::{RuleCache, RuleCacheHandle};
use stats::{StatsCollector, StatsHandle};
use api::create_router;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("goblin={}", config.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("GOBLIN DNS Redirector starting...");
    info!("Configuration: {:?}", config);

    // Initialize database
    let database = Arc::new(Database::new(&config.database_url()).await?);
    info!("Database connected: {}", config.database_path);

    // Run migrations
    database.run_migrations().await?;
    info!("Database migrations complete");

    // Initialize statistics collector
    let (stats_collector, stats_sender) = StatsCollector::new(database.clone(), 1000);
    let stats_handle = StatsHandle::new(stats_sender);
    info!("Statistics collector initialized");

    // Spawn statistics collector task
    tokio::spawn(async move {
        stats_collector.run().await;
    });

    // Initialize rule cache
    let rule_cache = Arc::new(RuleCache::new(database.clone(), config.rules_cache_ttl));
    let rule_cache_handle = RuleCacheHandle::new(rule_cache.clone());
    info!("Rule cache initialized (TTL: {}s)", config.rules_cache_ttl);

    // Spawn rule cache refresh loop
    tokio::spawn(async move {
        rule_cache.run_refresh_loop().await;
    });

    // Initialize DNS resolver
    let resolver = Arc::new(DnsResolver::new(&config.default_upstream).await?);
    info!("DNS resolver initialized");

    // Create DNS handler with rule cache and stats
    let handler = DnsHandler::new(resolver.clone())
        .with_rules(rule_cache_handle)
        .with_stats(stats_handle);

    // Create API router
    let app = create_router(database.clone(), config.api_cors_enabled);
    info!("API router created (CORS: {})", config.api_cors_enabled);

    // Spawn HTTP API server
    let api_addr = config.api_bind_address;
    tokio::spawn(async move {
        info!("Starting HTTP API server on {}", api_addr);
        let listener = TcpListener::bind(api_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Start DNS server (this blocks)
    info!("Starting DNS server on {}", config.dns_bind_address);
    let dns_server = DnsServer::new(config.dns_bind_address, config.dns_tcp_enabled, handler);

    // Run the DNS server (blocks until shutdown)
    dns_server.run().await?;

    Ok(())
}
