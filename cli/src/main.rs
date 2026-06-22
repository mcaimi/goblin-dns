mod client;
mod models;

use std::fmt;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

use client::GoblinClient;
use models::*;

#[derive(Parser, Debug)]
#[command(name = "goblin-cli", about = "CLI for Goblin DNS Smart Resolver")]
struct Cli {
    /// Goblin DNS API base URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    base_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check API health
    Health,

    /// Upstream resolver management
    #[command(subcommand)]
    Resolver(ResolverCommand),

    /// Priority rule management
    #[command(subcommand)]
    Rule(RuleCommand),

    /// Statistics
    #[command(subcommand)]
    Stats(StatsCommand),
}

#[derive(Subcommand, Debug)]
enum ResolverCommand {
    /// List all resolvers
    List,
    /// Get a resolver by ID
    Get { id: u64 },
    /// Create a new resolver
    Create {
        ip_address: String,
        domain: String,
    },
    /// Update a resolver
    Update {
        id: u64,
        #[arg(long)]
        ip: Option<String>,
        #[arg(long)]
        domain: Option<String>,
    },
    /// Delete a resolver
    Delete { id: u64 },
}

#[derive(Subcommand, Debug)]
enum RuleCommand {
    /// List all rules
    List,
    /// Get a rule by ID
    Get { id: u64 },
    /// Create a new rule
    Create {
        domain: String,
        target_ip: String,
        #[arg(long)]
        client_filter: Option<String>,
        #[arg(long)]
        active: Option<bool>,
    },
    /// Update a rule
    Update {
        id: u64,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        target_ip: Option<String>,
        #[arg(long)]
        client_filter: Option<String>,
        #[arg(long)]
        clear_client_filter: bool,
        #[arg(long)]
        active: Option<bool>,
    },
    /// Toggle a rule active state
    Toggle { id: u64 },
    /// Delete a rule
    Delete { id: u64 },
}

#[derive(Subcommand, Debug)]
enum StatsCommand {
    /// List DNS query statistics
    List {
        #[arg(long, default_value = "100")]
        limit: u64,
        #[arg(long, default_value = "0")]
        offset: u64,
    },
    /// Get statistics for a specific rule
    ForRule { rule_id: u64 },
    /// Get statistics summary
    Summary,
}

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CliError {}

fn print_resolvers(data: Vec<Resolver>) {
    if data.is_empty() {
        println!("No resolvers found.");
        return;
    }
    println!(
        "{:>5}  {:<18}  {:<25}  {:<22}  {:<22}",
        "ID", "IP Address", "Domain", "Created", "Updated"
    );
    println!(
        "{:-<5}-+-{:-<18}-+-{:-<25}-+-{:-<22}-+-{:-<22}",
        "", "", "", "", ""
    );
    for r in &data {
        println!(
            "{:>5}  {:<18}  {:<25}  {:<22}  {:<22}",
            r.id,
            r.ip_address,
            r.domain,
            format_timestamp(r.created_at),
            format_timestamp(r.updated_at)
        );
    }
}

fn print_resolver(r: Resolver) {
    println!("ID:            {}", r.id);
    println!("IP Address:    {}", r.ip_address);
    println!("Domain:        {}", r.domain);
    println!("Created:       {}", format_timestamp(r.created_at));
    println!("Updated:       {}", format_timestamp(r.updated_at));
}

fn print_rules(data: Vec<Rule>) {
    if data.is_empty() {
        println!("No rules found.");
        return;
    }
    println!(
        "{:>5}  {:<30}  {:<18}  {:<8}  {:<22}  {:<22}",
        "ID", "Domain", "Target IP", "Active", "Created", "Updated"
    );
    println!(
        "{:-<5}-+-{:-<30}-+-{:-<18}-+-{:-<8}-+-{:-<22}-+-{:-<22}",
        "", "", "", "", "", ""
    );
    for r in &data {
        let client_filter = r
            .client_filter
            .as_deref()
            .unwrap_or("all");
        println!(
            "{:>5}  {:<30}  {:<18}  {:<8}  {:<22}  {:<22}",
            r.id,
            format!("{} ({})", r.domain, client_filter),
            r.target_ip,
            r.active,
            format_timestamp(r.created_at),
            format_timestamp(r.updated_at)
        );
    }
}

fn print_rule(r: Rule) {
    println!("ID:             {}", r.id);
    println!("Domain:         {}", r.domain);
    println!(
        "Client Filter:  {}",
        r.client_filter.as_deref().unwrap_or("all")
    );
    println!("Target IP:      {}", r.target_ip);
    println!("Active:         {}", r.active);
    println!("Created:        {}", format_timestamp(r.created_at));
    println!("Updated:        {}", format_timestamp(r.updated_at));
}

fn print_stats(data: Vec<StatEntry>) {
    if data.is_empty() {
        println!("No statistics found.");
        return;
    }
    println!(
        "{:>5}  {:>16}  {:<18}  {:<30}  {:<18}  {:<18}  {:<22}",
        "ID", "Rule ID", "Client IP", "Requested Domain", "Original IP", "Overridden IP", "Timestamp"
    );
    println!(
        "{:-<5}-+-{:-<16}-+-{:-<18}-+-{:-<30}-+-{:-<18}-+-{:-<18}-+-{:-<22}",
        "", "", "", "", "", "", ""
    );
    for s in &data {
        println!(
            "{:>5}  {:>16}  {:<18}  {:<30}  {:<18}  {:<18}  {:<22}",
            s.id,
            s.rule_id,
            s.client_ip,
            s.requested_domain,
            s.original_ip.as_deref().unwrap_or("-"),
            s.overridden_ip,
            format_timestamp(s.timestamp)
        );
    }
}

fn print_summary(data: StatsSummary) {
    println!("Total Hits:     {}", data.total_hits);
    println!("Unique Clients: {}", data.unique_clients);
    println!("Active Rules:   {}", data.active_rules);
}

fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return String::from("N/A");
    }
    let dt = DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(ts));
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = GoblinClient::new(&cli.base_url);

    match cli.command {
        Commands::Health => {
            // call health endpoint
            // fails on error directly
            client.health_check().await.map_err(|e| CliError(e.to_string()))?;
            
            // api is LIVE, everything is ok
            println!("Goblin DNS API is healthy.");
            return Ok(());
        }
        Commands::Resolver(cmd) => match cmd {
            ResolverCommand::List => {
                let resp = client.list_resolvers().await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_resolvers(resp.data);
            }
            ResolverCommand::Get { id } => {
                let resp = client.get_resolver(id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_resolver(resp.data);
            }
            ResolverCommand::Create {
                ip_address,
                domain,
            } => {
                let resp = client.create_resolver(&ip_address, &domain).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Created resolver (ID: {})", resp.data["id"]);
            }
            ResolverCommand::Update {
                id,
                ip,
                domain,
            } => {
                let resp = client
                    .update_resolver(id, ip.as_deref(), domain.as_deref())
                    .await
                    .map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Resolver {} updated.", id);
            }
            ResolverCommand::Delete { id } => {
                let resp = client.delete_resolver(id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Resolver {} deleted.", id);
            }
        },
        Commands::Rule(cmd) => match cmd {
            RuleCommand::List => {
                let resp = client.list_rules().await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_rules(resp.data);
            }
            RuleCommand::Get { id } => {
                let resp = client.get_rule(id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_rule(resp.data);
            }
            RuleCommand::Create {
                domain,
                target_ip,
                client_filter,
                active,
            } => {
                let resp = client.create_rule(&domain, &target_ip, client_filter.as_deref(), active)
                    .await
                    .map_err(|e| CliError(e.to_string()))?;
                
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Created rule (ID: {})", resp.data["id"]);
            }
            RuleCommand::Update {
                id,
                domain,
                target_ip,
                client_filter,
                clear_client_filter,
                active,
            } => {
                let cf = if clear_client_filter {
                    None
                } else {
                    client_filter.as_deref()
                };
                let resp = client
                    .update_rule(id, domain.as_deref(), target_ip.as_deref(), Some(cf), active)
                    .await
                    .map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Rule {} updated.", id);
            }
            RuleCommand::Toggle { id } => {
                let resp = client.toggle_rule(id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Rule {} toggled.", id);
            }
            RuleCommand::Delete { id } => {
                let resp = client.delete_rule(id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                println!("Rule {} deleted.", id);
            }
        },
        Commands::Stats(cmd) => match cmd {
            StatsCommand::List { limit, offset } => {
                let resp = client.list_stats(limit, offset).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_stats(resp.data);
            }
            StatsCommand::ForRule { rule_id } => {
                let resp = client.stats_for_rule(rule_id).await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_stats(resp.data);
            }
            StatsCommand::Summary => {
                let resp = client.stats_summary().await.map_err(|e| CliError(e.to_string()))?;
                println!("API Call Returns OK: [{}]", resp.success);
                print_summary(resp.data);
            }
        },
    };

    Ok(())
}
