# Goblin DNS Redirector - Backend Architecture

A high-performance, async DNS redirector resolver built in Rust. It intercepts standard DNS queries, matches them against configurable priority rules (by domain and client IP/CIDR), and returns overridden IP addresses. Non-matching queries are forwarded to configurable upstream resolvers.

## Table of Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Architecture Overview](#architecture-overview)
- [Module Details](#module-details)
  - [Configuration (`config.rs`)](#configuration-configrs)
  - [Database Layer (`db/`)](#database-layer-db)
  - [DNS Server (`dns/`)](#dns-server-dns)
  - [Rule Engine (`rules/`)](#rule-engine-rules)
  - [Statistics (`stats/`)](#statistics-stats)
  - [REST API (`api/`)](#rest-api-api)
- [DNS Query Processing Flow](#dns-query-processing-flow)
- [Application Bootstrap](#application-bootstrap)
- [Database Schema](#database-schema)
- [Design Patterns](#design-patterns)
- [Dependencies](#dependencies)

## Quick Start

```bash
cd backend
cargo run --release

# Initialize database (first run)
sqlite3 goblin.db < migrations/001_initial.sql
```

The DNS server listens on `0.0.0.0:53` (UDP/TCP). The REST API listens on `0.0.0.0:8080`.

```bash
# Test with dig
dig @127.0.0.1 example.com
curl http://localhost:8080/api/v1/health

# View complete API documentation in browser
open http://localhost:8080/api/v1/docs
```

## Project Structure

```
backend/
├── Cargo.toml
├── Containerfile
├── migrations/
│   └── 001_initial.sql
└── src/
    ├── main.rs           # Entry point, signal handling, task orchestration
    ├── config.rs         # CLI args + environment-based configuration
    ├── db/
    │   ├── mod.rs        # Module re-exports
    │   ├── models.rs     # Domain models: UpstreamResolver, PriorityRule, Statistic
    │   └── queries.rs    # Database struct with all SQL operations
    ├── dns/
    │   ├── mod.rs        # Module re-exports
    │   ├── server.rs     # Hickory DNS server (UDP + TCP) wrapper
    │   ├── handler.rs    # DNS request handler implementing RequestHandler trait
    │   └── resolver.rs   # Upstream DNS resolution wrapper
    ├── rules/
    │   ├── mod.rs        # Module re-exports
    │   ├── cache.rs      # In-memory rule cache with periodic refresh
    │   └── matcher.rs    # Domain + client IP matching logic
    ├── stats/
    │   ├── mod.rs        # Module re-exports
    │   └── collector.rs  # Async stats collector via mpsc channel
    └── api/
        ├── mod.rs        # Module re-exports
        ├── routes.rs     # Axum router + CORS setup
        ├── models.rs     # Request/Response DTOs
        └── handlers.rs   # REST endpoint handlers
```

## Architecture Overview

The system follows a **layered architecture** with clear separation of concerns:

```
                    ┌──────────────┐
    DNS queries     │  DNS Server  │  (port 53, UDP/TCP)
    ──────────────► │  (Hickory)   │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │   Request    │
                    │    Handler   │
                    └──┬───┬───┬───┘
                       │   │   │
          ┌────────────┘   │   └──────────────┐
          ▼                ▼                   ▼
   ┌──────────┐    ┌───────────┐      ┌──────────────┐
   │ Rule     │    │ Upstream  │      │  Stats       │
   │ Cache    │    │ Resolver  │      │  Collector   │
   │ Match?   │    │ (Hickory) │      │  (mpsc + SQLite)│
   └──────────┘    └───────────┘      └──────────────┘
                                                   
                    ┌──────────────┐
                    │   SQLite     │
                    │  (WAL mode)  │
                    └──────────────┘

                    ┌──────────────┐
    REST API        │    Axum      │  (port 8080)
    ──────────────► │  Web Server  │
                    └──────────────┘
```

The DNS server handles incoming queries on port 53, the rule engine determines whether to override resolution, the upstream resolver handles normal queries, and the stats collector asynchronously persists hit data. Concurrently, the REST API serves management endpoints on port 8080.

## Module Details

### Configuration (`config.rs`)

Configuration is parsed via `clap` derive macros with environment variable fallback:

| Field | Env Variable | Default |
|---|---|---|
| `dns_bind_address` | `DNS_BIND_ADDRESS` | `0.0.0.0:53` |
| `dns_tcp_enabled` | `DNS_TCP_ENABLED` | `true` |
| `api_bind_address` | `API_BIND_ADDRESS` | `0.0.0.0:8080` |
| `api_cors_enabled` | `API_CORS_ENABLED` | `true` |
| `database_path` | `DATABASE_PATH` | `./goblin.db` |
| `rules_cache_ttl` | `RULES_CACHE_TTL` | `60` (seconds) |
| `log_level` | `LOG_LEVEL` | `info` |
| `default_upstream` | `DEFAULT_UPSTREAM` | `8.8.8.8:53` |

```rust
struct Config {
    dns_bind_address: SocketAddr,
    dns_tcp_enabled: bool,
    api_bind_address: SocketAddr,
    api_cors_enabled: bool,
    database_path: String,
    rules_cache_ttl: u64,
    log_level: String,
    default_upstream: String,
}
```

`CliArgs` wraps the clap parser (all fields `Option<T>`), resolved via `Config::from_env()` with defaults applied. The database path is formatted into a `sqlx://` URL via `database_url()`.

### Database Layer (`db/`)

**`models.rs`** defines three domain structs:

```rust
struct UpstreamResolver {
    id: i64,
    ip_address: String,
    domain: String,
    created_at: i64,
    updated_at: i64,
}

struct PriorityRule {
    id: i64,
    domain: String,
    client_filter: Option<String>,  // None = all clients; exact IP or CIDR otherwise
    target_ip: String,              // IP to return in DNS response
    active: bool,
    created_at: i64,
    updated_at: i64,
}

struct Statistic {
    id: i64,
    rule_id: i64,
    client_ip: String,
    requested_domain: String,
    original_ip: Option<String>,    // IP returned by upstream resolver
    overridden_ip: String,          // IP from the matched rule
    timestamp: i64,
}

struct StatsSummary {
    total_hits: i64,
    unique_clients: i64,
    active_rules: i64,
}
```

**`queries.rs`** provides the `Database` struct wrapping an `SqlitePool`:

```rust
struct Database {
    pool: SqlitePool,  // SQLite connection pool with WAL mode
}
```

Key methods: `run_migrations()`, CRUD for resolvers and rules (including `toggle_rule()`), `record_stat()`, paginated stats queries (`get_all_stats`), per-rule stats (`get_stats_for_rule`), and `get_stats_summary()` with aggregation.

### DNS Server (`dns/`)

**`server.rs`** — Wraps Hickory's `ServerFuture`:

```rust
struct DnsServer {
    bind_address: SocketAddr,
    tcp_enabled: bool,
    handler: DnsHandler,
}
```

`run()` registers UDP and optionally TCP sockets, then blocks.

**`resolver.rs`** — Wrapper around Hickory's `TokioAsyncResolver`:

```rust
struct DnsResolver {
    resolver: TokioAsyncResolver,  // Hickory async resolver configured with custom upstream
}
```

`new(upstream_addr)` configures a custom resolver using a single upstream (e.g., `8.8.8.8:53`). Provides `resolve_a(domain)` and `resolve_aaaa(domain)` convenience methods.

**`handler.rs`** — Core request handler implementing `hickory_server::server::RequestHandler`:

```rust
struct DnsHandler {
    resolver: Arc<DnsResolver>,
    rule_cache: Option<RuleCacheHandle>,
    stats_handle: Option<StatsHandle>,
}
```

Uses a **builder pattern** for chaining:

```rust
DnsHandler::new(resolver)
    .with_rules(rule_cache_handle)
    .with_stats(stats_handle)
```

The `handle_query()` method checks `rule_cache` first — if a rule matches, it returns the `target_ip`; otherwise it delegates to `resolver`. All returned records use a hardcoded TTL of 300 seconds.

### Rule Engine (`rules/`)

**`matcher.rs`** — Pure matching functions with specificity priority:

```rust
fn client_matches(&rule, client_ip) -> bool
fn domain_matches(&rule, requested_domain) -> bool
fn find_matching_rule(&rules, domain, client_ip) -> Option<&PriorityRule>
```

`domain_matches()` does case-insensitive exact matching, stripping trailing dots.

`client_matches()` handles three modes: `None` (all clients), exact IP string comparison, and CIDR matching via `ipnetwork`.

`find_matching_rule()` implements a **specificity priority** sort:

1. Filter: `active == true` AND `domain_matches`
2. Filter by `client_matches`
3. Sort by specificity: **exact IP > CIDR (longer prefix wins) > no filter**
4. Return the most specific match (first in sorted order)

**`cache.rs`** — In-memory cache with periodic DB refresh:

```rust
struct RuleCache {
    db: Arc<Database>,
    rules: Arc<RwLock<Vec<PriorityRule>>>,
    refresh_interval: Duration,
}

struct RuleCacheHandle {
    cache: Arc<RuleCache>,
}
```

`RuleCache` uses `Arc<RwLock<Vec<PriorityRule>>>` for concurrent reads with infrequent writes. `RuleHandle` is a cloneable wrapper hidden behind `Arc`. The `run_refresh_loop()` task calls `refresh()` (fetching active rules from DB) every `RULES_CACHE_TTL` seconds.

### Statistics (`stats/`)

**`collector.rs`** — Decouples DNS query handling from database writes via an `mpsc` channel:

```rust
struct StatEvent {
    rule_id: i64,
    client_ip: String,
    requested_domain: String,
    original_ip: Option<String>,
    overridden_ip: String,
}

struct StatsCollector {
    db: Arc<Database>,
    receiver: mpsc::Receiver<StatEvent>,
}

struct StatsHandle {
    sender: mpsc::Sender<StatEvent>,
}
```

`StatsHandle::try_record()` sends non-blocking (drops events when the 1000-item buffer is full). `StatsCollector::run()` is a spawned async task that receives events and persists them via `db.record_stat()`.

### REST API (`api/`)

**`routes.rs`** — Sets up the Axum router with all endpoints under `/api/v1`:

| Method | Path | Handler |
|---|---|---|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/docs` | Complete API documentation (HTML) |
| GET/POST/GET/PUT/DELETE | `/api/v1/resolvers` | Upstream resolver CRUD |
| GET/POST/GET/PUT/PATCH/DELETE | `/api/v1/rules` | Priority rule CRUD + toggle |
| GET | `/api/v1/stats` | Paginated statistics |
| GET | `/api/v1/stats/rules/:rule_id` | Per-rule statistics |
| GET | `/api/v1/stats/summary` | Aggregated summary |

A generic `ApiResponse<T>` wrapper provides consistent JSON responses: `{"success": bool, "data": T?, "error": String?}`. CORS is configurable via `API_CORS_ENABLED`.

**API Documentation** — Visit `http://localhost:8080/api/v1/docs` in your browser for interactive HTML documentation with detailed endpoint descriptions, request/response examples, and a quick start guide.

**`models.rs`** defines request/response DTOs including `CreateRuleRequest`, `UpdateRuleRequest` (with `Option<Option<T>>` for optional fields), `StatsQuery`, and `StatsSummaryResponse`.

## DNS Query Processing Flow

The complete lifecycle of a DNS query through the system:

```mermaid
graph TB
    C[Client DNS Query] -->|"UDP:53 or TCP:53"| S[DnsServer]

    subgraph "dns/handler.rs"
        S --> H[DnsHandler::handle_request]
        H --> Parse[Parse: domain, record type, client IP]
        Parse -->|"Only A/AAAA"| MatchRule{Match Rule?}
        Parse -->|"Other type"| NotImp[NotImp Response]
    end

    subgraph "rules/"
        MatchRule -->|"Yes"| Fetch[RuleCache::get_rules]
        Fetch --> Sort[Specificity Sort]
        Sort --> Best[Best Matching Rule]
        Best --> MatchRule
    end

    subgraph "Override Path"
        MatchRule -->|"Rule matched"| Build[Build Record from target_ip]
        Build -->|"Resolve original for stats"| UR[Upstream Resolver lookup]
        UR -->|"Optional"| Stats[StatsHandle::try_record]
        Stats -->|"mpsc channel"| SC[StatsCollector::run]
    end

    subgraph "Normal Path"
        MatchRule -->|"No match"| Default[Upstream Resolver::resolve]
        Default -->|"A/AAAA lookup"| Resp[Build A/AAAA Record]
    end

    Build -->|"A/AAAA, TTL=300"| Resp[Build DNS Response]
    Resp -->|"NoError / NXDomain / ServFail"| C
```

Step-by-step:

1. **Packet arrival**: Client sends DNS query to bind address over UDP or TCP
2. **Server setup**: `DnsServer` registers socket with `ServerFuture::new(handler)`
3. **Handler dispatch**: `DnsHandler::handle_request()` is called by Hickory's server loop
4. **Query parsing**: Extract domain, record type, and client IP from the request
5. **Type check**: Only `A` and `AAAA` queries are handled; others receive `NotImp`
6. **Rule lookup**: Fetch rules from cache, filter by `active` + domain match + client match, sort by specificity
7. **Override path** (if rule matches): Parse `target_ip`, optionally resolve original IP for stats, send `StatEvent` via channel, return record
8. **Normal path** (if no match): Forward to `DnsResolver` which queries the upstream DNS
9. **Response**: `MessageResponseBuilder` constructs the DNS response with `NoError`, `NXDomain`, `ServFail`, or `NotImp`

## Application Bootstrap

`main.rs` orchestrates all components with a shared `CancellationToken` for graceful shutdown:

```mermaid
sequenceDiagram
    participant M as main()
    participant C as Config::from_env()
    participant L as tracing_subscriber
    participant DB as Database (SQLite)
    participant S as StatsCollector
    participant RC as RuleCache
    participant R as DnsResolver
    participant H as DnsHandler
    participant A as Axum Router
    participant D as DnsServer
    participant Sig as Ctrl-C / SIGTERM

    M->>C: Parse config
    M->>L: Init EnvFilter
    M->>DB: new(url) + run_migrations()
    M->>S: StatsCollector::new(db, 1000)
    Note over S: Returns (collector, handle)
    M->>M: tokio::spawn(collector.run())

    M->>RC: RuleCache::new(db, ttl)
    Note over RC: Uses RwLock<Vec<PriorityRule>>
    M->>M: tokio::spawn(cache.run_refresh_loop())

    M->>R: DnsResolver::new(upstream)
    M->>H: DnsHandler::new(resolver)
    H->>H: .with_rules(cache_handle)
    H->>H: .with_stats(stats_handle)

    M->>A: create_router(db, cors)
    M->>M: tokio::spawn(axum::serve())

    M->>Sig: Spawn ctrl_c + SIGTERM handler
    Sig->>M: cancel_token.cancel()

    M->>D: DnsServer::new(addr, tcp, handler)
    D->>D: run() blocks
    Note over D,Sig: tokio::select! between
    Note over D,Sig: dns_server.run() and shutdown
```

Bootstrap order:

1. Parse config (`Config::from_env()`)
2. Initialize `tracing_subscriber` with `EnvFilter`
3. Connect SQLite, enable WAL mode, run migrations
4. Create `StatsCollector` (returns collector + `StatsHandle`), spawn as task
5. Create `RuleCache` with refresh interval, spawn refresh loop task
6. Create `DnsResolver` with upstream config
7. Build `DnsHandler` via builder pattern, chaining in rules and stats
8. Create Axum router with all routes + optional CORS
9. Spawn HTTP server on `api_bind_address`
10. Set up Ctrl-C and SIGTERM handlers that call `cancel()` on the shared token
11. Run `DnsServer::run()` (blocking), selected against shutdown signal

## Database Schema

**upstream_resolvers** — Maps domains to upstream DNS server IPs:

| Column | Type | Notes |
|---|---|---|
| id | INTEGER | PK, autoincrement |
| ip_address | TEXT | Upstream resolver IP |
| domain | TEXT | Domain this resolver handles |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |

*Constraint: `UNIQUE(ip_address, domain)`*

**priority_rules** — Overrides for DNS resolution:

| Column | Type | Notes |
|---|---|---|
| id | INTEGER | PK, autoincrement |
| domain | TEXT | Domain to match |
| client_filter | TEXT | NULL=all, else exact IP or CIDR |
| target_ip | TEXT | IP to return |
| active | INTEGER | 0=inactive, 1=active |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |

*Constraint: `UNIQUE(domain, client_filter)`*

**statistics** — Tracks DNS query hits against rules:

| Column | Type | Notes |
|---|---|---|
| id | INTEGER | PK, autoincrement |
| rule_id | INTEGER | FK -> priority_rules.id |
| client_ip | TEXT | Query source IP |
| requested_domain | TEXT | Query domain |
| original_ip | TEXT | Upstream resolver response |
| overridden_ip | TEXT | Rule target IP |
| timestamp | INTEGER | Query time |

## Design Patterns

| Pattern | Usage |
|---|---|
| **Builder pattern** | `DnsHandler::new(resolver).with_rules(handle).with_stats(handle)` for composed request handling |
| **Channel-based decoupling** | `StatsHandle` / `StatsCollector` use `mpsc` to avoid blocking DNS handler on DB writes |
| **Handle pattern** | `RuleCacheHandle` and `StatsHandle` hide internal `Arc` complexity, are `Clone`, and can be shared across tasks |
| **RwLock for concurrent reads** | `RuleCache` uses `Arc<RwLock<Vec<PriorityRule>>>` allowing concurrent rule lookups with infrequent background refresh |
| **CancellationToken for shutdown** | Single `CancellationToken` shared across all spawned tasks; `tokio::select!` listens for cancel signal |
| **WAL mode** | SQLite operates in Write-Ahead Logging mode for better concurrent read/write performance |

## Class Diagram

```mermaid
classDiagram
    class Config {
        +dns_bind_address: SocketAddr
        +dns_tcp_enabled: bool
        +api_bind_address: SocketAddr
        +api_cors_enabled: bool
        +database_path: String
        +rules_cache_ttl: u64
        +log_level: String
        +default_upstream: String
        +from_env() Config
        +database_url() String
    }

    class CliArgs {
        +dns_bind_address: Option~SocketAddr~
        +dns_tcp_enabled: Option~bool~
        +api_bind_address: Option~SocketAddr~
        +api_cors_enabled: Option~bool~
        +database_path: Option~String~
        +rules_cache_ttl: Option~u64~
        +log_level: Option~String~
        +default_upstream: Option~String~
    }

    Config --> CliArgs : "parsed from"

    class Database {
        -pool: SqlitePool
        +new(url) Database
        +run_migrations()
        +get_all_resolvers() Vec~UpstreamResolver~
        +create_resolver(ip, domain) i64
        +update_resolver(id, ip, domain) bool
        +delete_resolver(id) bool
        +get_all_rules() Vec~PriorityRule~
        +get_active_rules() Vec~PriorityRule~
        +create_rule(domain, client_filter, target_ip, active) i64
        +update_rule(id, domain, client_filter, target_ip, active) bool
        +toggle_rule(id) bool
        +delete_rule(id) bool
        +record_stat(rule_id, client_ip, domain, original_ip, overridden_ip)
        +get_all_stats(limit, offset) Vec~Statistic~
        +get_stats_for_rule(rule_id) Vec~Statistic~
        +get_stats_summary() StatsSummary
    }

    class PriorityRule {
        +id: i64
        +domain: String
        +client_filter: Option~String~
        +target_ip: String
        +active: bool
        +created_at: i64
        +updated_at: i64
    }

    class UpstreamResolver {
        +id: i64
        +ip_address: String
        +domain: String
        +created_at: i64
        +updated_at: i64
    }

    class Statistic {
        +id: i64
        +rule_id: i64
        +client_ip: String
        +requested_domain: String
        +original_ip: Option~String~
        +overridden_ip: String
        +timestamp: i64
    }

    class StatsSummary {
        +total_hits: i64
        +unique_clients: i64
        +active_rules: i64
    }

    class DnsServer {
        -bind_address: SocketAddr
        -tcp_enabled: bool
        -handler: DnsHandler
        +new(addr, tcp, handler)
        +run()
    }

    class DnsHandler {
        -resolver: Arc~DnsResolver~
        -rule_cache: Option~RuleCacheHandle~
        -stats_handle: Option~StatsHandle~
        +new(resolver) DnsHandler
        +with_rules(handle) DnsHandler
        +with_stats(handle) DnsHandler
        +handle_query(request) Vec~Record~
        +handle_request(request, handle) ResponseInfo
    }

    class DnsResolver {
        -resolver: TokioAsyncResolver
        +new(upstream) Self
        +resolve(domain, type) Vec~IpAddr~
        +resolve_a(domain) Vec~IpAddr~
        +resolve_aaaa(domain) Vec~IpAddr~
    }

    class RuleCache {
        -db: Arc~Database~
        -rules: Arc~RwLock~Vec~PriorityRule~~~
        -refresh_interval: Duration
        +new(db, ttl) Self
        +get_rules() Vec~PriorityRule~
        +refresh() Result~usize~~
        +run_refresh_loop()
    }

    class RuleCacheHandle {
        -cache: Arc~RuleCache~
        +get_rules() Vec~PriorityRule~
        +refresh() Result~usize~~
    }

    class RuleMatcher

    class StatEvent {
        +rule_id: i64
        +client_ip: String
        +requested_domain: String
        +original_ip: Option~String~
        +overridden_ip: String
    }

    class StatsCollector {
        -db: Arc~Database~
        -receiver: mpsc::Receiver~StatEvent~
        +new(db, buffer) (Self, Sender~StatEvent~~)
        +run()
    }

    class StatsHandle {
        -sender: mpsc::Sender~StatEvent~
        +record(event) bool
        +try_record(event) bool
    }

    class AppState {
        +db: Arc~Database~
    }

    class Routes {
        +create_router(db, cors) Router
    }

    class CreateRuleRequest {
        +domain: String
        +client_filter: Option~String~
        +target_ip: String
        +active: bool
    }

    class UpdateRuleRequest {
        +domain: Option~String~
        +client_filter: Option~Option~String~~
        +target_ip: Option~String~
        +active: Option~bool~
    }

    class StatsSummaryResponse {
        +total_hits: i64
        +unique_clients: i64
        +active_rules: i64
    }

    class ApiResponse~T~ {
        +success: bool
        +data: Option~T~
        +error: Option~String~
        +success(data) ApiResponse~T~~
        +error(msg) ApiResponse~T~~
    }

    Database --> PriorityRule : "CRUD"
    Database --> UpstreamResolver : "CRUD"
    Database --> Statistic : "record/query"
    Database --> StatsSummary : "aggregate"
    PriorityRule ..> RuleMatcher : "matched by"

    DnsServer --> DnsHandler : "owns"
    DnsHandler --> DnsResolver : "owns"
    DnsHandler --> RuleCacheHandle : "uses (optional)"
    DnsHandler --> StatsHandle : "uses (optional)"

    RuleCache --> Database : "reads from"
    RuleCacheHandle --> RuleCache : "wraps"

    RuleCache -right-> PriorityRule : "caches"
    StatsCollector --> Database : "writes to"
    StatsCollector ..> StatEvent : "receives via channel"
    StatsHandle --> StatEvent : "sends via channel"

    Routes --> AppState : "uses state"
    AppState --> Database : "owns"
    Api~~> ApiResponse : "wraps"
```

## Dependencies

| Category | Crate | Purpose |
|---|---|---|
| **Async Runtime** | `tokio`, `tokio-util` | Async runtime, CancellationToken |
| **DNS** | `hickory-server`, `hickory-resolver`, `hickory-proto` | DNS server protocol, upstream resolution |
| **Web** | `axum`, `tower`, `tower-http` | HTTP API, CORS, tracing middleware |
| **Database** | `sqlx` (sqlite, tokio, migrate) | Compile-time checked SQL + migrations |
| **Serialization** | `serde`, `serde_json` | JSON serialization |
| **Networking** | `ipnetwork` | CIDR network matching |
| **Error Handling** | `anyhow`, `thiserror` | Error chaining + ergonomic errors |
| **Logging** | `tracing`, `tracing-subscriber` (env-filter) | Structured logging |
| **CLI** | `clap` (derive) | Argument parsing with env var fallback |
| **Utilities** | `chrono` (serde), `async-trait` | Timestamps, async trait support |
