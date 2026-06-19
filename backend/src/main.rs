mod config;
mod db;
mod dns;
mod rules;
mod stats;
mod api;

use config::Config;
use db::Database;
use dns::{DnsResolver, DnsServer, DnsHandler, ResolverCache, ResolverCacheHandle};
use rules::{RuleCache, RuleCacheHandle};
use stats::{StatsCollector, StatsHandle};
use api::create_router;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;

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

    // Create cancellation token for graceful shutdown
    let shutdown_token = CancellationToken::new();

    // Initialize statistics collector
    let (stats_collector, stats_sender) = StatsCollector::new(database.clone(), 1000);
    let stats_handle = StatsHandle::new(stats_sender);
    info!("Statistics collector initialized");

    // Spawn statistics collector task
    let stats_shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = stats_collector.run() => {},
            _ = stats_shutdown.cancelled() => {
                info!("Statistics collector shutting down");
            }
        }
    });

    // Initialize rule cache
    let rule_cache = Arc::new(RuleCache::new(database.clone(), config.rules_cache_ttl));
    let rule_cache_handle = RuleCacheHandle::new(rule_cache.clone());
    info!("Rule cache initialized (TTL: {}s)", config.rules_cache_ttl);

    // Spawn rule cache refresh loop
    let cache_shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = rule_cache.run_refresh_loop() => {},
            _ = cache_shutdown.cancelled() => {
                info!("Rule cache refresh loop shutting down");
            }
        }
    });

    // Initialize resolver cache
    let resolver_cache = Arc::new(ResolverCache::new(database.clone(), config.rules_cache_ttl));
    let resolver_cache_handle = ResolverCacheHandle::new(resolver_cache.clone());
    info!("Resolver cache initialized (TTL: {}s)", config.rules_cache_ttl);

    // Spawn resolver cache refresh loop
    let resolver_shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = resolver_cache.run_refresh_loop() => {},
            _ = resolver_shutdown.cancelled() => {
                info!("Resolver cache refresh loop shutting down");
            }
        }
    });

    // Create fallback DNS resolver (in case no upstream resolvers are configured)
    let fallback_resolver = Arc::new(DnsResolver::new(&config.default_upstream).await?);
    info!("Fallback DNS resolver initialized: {}", config.default_upstream);

    // Create DNS handler with resolver cache, rules, and stats
    let handler = DnsHandler::with_resolver_cache(resolver_cache_handle)
        .with_rules(rule_cache_handle)
        .with_stats(stats_handle);

    // Create API router
    let app = create_router(database.clone(), config.api_cors_enabled);
    info!("API router created (CORS: {})", config.api_cors_enabled);

    // Spawn HTTP API server
    let api_addr = config.api_bind_address;
    let api_shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        info!("Starting HTTP API server on {}", api_addr);
        let listener = TcpListener::bind(api_addr).await.unwrap();
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                api_shutdown.cancelled().await;
                info!("HTTP API server shutting down");
            })
            .await
            .unwrap();
    });

    // Start DNS server (this blocks)
    info!("Starting DNS server on {}", config.dns_bind_address);
    let dns_server = DnsServer::new(config.dns_bind_address, config.dns_tcp_enabled, handler);

    // Spawn signal handler for graceful shutdown
    let signal_shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received SIGINT, initiating graceful shutdown");
                signal_shutdown.cancel();
            }
            Err(err) => {
                tracing::error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    // Handle SIGTERM on Unix systems
    #[cfg(unix)]
    {
        let signal_shutdown = shutdown_token.clone();
        tokio::spawn(async move {
            match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                Ok(mut stream) => {
                    stream.recv().await;
                    info!("Received SIGTERM, initiating graceful shutdown");
                    signal_shutdown.cancel();
                }
                Err(err) => {
                    tracing::error!("Unable to listen for SIGTERM: {}", err);
                }
            }
        });
    }

    // Run the DNS server with graceful shutdown
    tokio::select! {
        result = dns_server.run() => {
            if let Err(e) = result {
                tracing::error!("DNS server error: {}", e);
            }
        }
        _ = shutdown_token.cancelled() => {
            info!("DNS server shutting down");
        }
    }

    info!("Graceful shutdown complete");
    Ok(())
}
