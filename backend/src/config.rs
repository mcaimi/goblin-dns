use clap::Parser;
use std::net::SocketAddr;
use std::env;

#[derive(Debug, Clone, Parser)]
#[command(name = "GOBLIN")]
#[command(about = "A DNS redirector for security researchers", long_about = None)]
pub struct CliArgs {
    /// DNS server bind address
    #[arg(long)]
    pub dns_bind_address: Option<SocketAddr>,

    /// Enable TCP DNS server in addition to UDP
    #[arg(long)]
    pub dns_tcp_enabled: Option<bool>,

    /// HTTP API server bind address
    #[arg(long)]
    pub api_bind_address: Option<SocketAddr>,

    /// Enable CORS for the HTTP API
    #[arg(long)]
    pub api_cors_enabled: Option<bool>,

    /// SQLite database path
    #[arg(long)]
    pub database_path: Option<String>,

    /// Rules cache TTL in seconds
    #[arg(long)]
    pub rules_cache_ttl: Option<u64>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dns_bind_address: SocketAddr,
    pub dns_tcp_enabled: bool,
    pub api_bind_address: SocketAddr,
    pub api_cors_enabled: bool,
    pub database_path: String,
    pub rules_cache_ttl: u64,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        let cli = CliArgs::parse();

        Config {
            dns_bind_address: cli.dns_bind_address.or_else(|| env::var("DNS_BIND_ADDRESS").ok().and_then(|s| s.parse().ok())).unwrap_or_else(|| "0.0.0.0:53".parse().unwrap()),
            dns_tcp_enabled: cli.dns_tcp_enabled.or_else(|| env::var("DNS_TCP_ENABLED").ok().and_then(|s| s.parse().ok())).unwrap_or(true),
            api_bind_address: cli.api_bind_address.or_else(|| env::var("API_BIND_ADDRESS").ok().and_then(|s| s.parse().ok())).unwrap_or_else(|| "0.0.0.0:8080".parse().unwrap()),
            api_cors_enabled: cli.api_cors_enabled.or_else(|| env::var("API_CORS_ENABLED").ok().and_then(|s| s.parse().ok())).unwrap_or(true),
            database_path: cli.database_path.or_else(|| env::var("DATABASE_PATH").ok()).unwrap_or_else(|| "./goblin.db".to_string()),
            rules_cache_ttl: cli.rules_cache_ttl.or_else(|| env::var("RULES_CACHE_TTL").ok().and_then(|s| s.parse().ok())).unwrap_or(60),
            log_level: cli.log_level.or_else(|| env::var("LOG_LEVEL").ok()).unwrap_or_else(|| "info".to_string()),
        }
    }

    pub fn database_url(&self) -> String {
        format!("sqlite:{}", self.database_path)
    }
}
