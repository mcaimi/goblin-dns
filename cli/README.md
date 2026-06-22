# goblin-cli

CLI tool for managing the Goblin DNS Redirector backend.

## Architecture

```
┌─────────────┐     HTTP/JSON (REST)      ┌──────────────────┐
│  goblin-cli  │ ───────────────────────► │  Goblin Backend   │
│  (this tool) │  /api/v1/{resolvers|     │  Rust + Axum      │
│  clap +      │    rules|stats}           │  SQLite           │
│  reqwest     │ ◄─────────────────────── │  Tokio async      │
└─────────────┘        JSON responses     └──────────────────┘
```

### Module layout

- **`main.rs`** — CLI definition and output formatting
  - Parses arguments via `clap` derive macros
  - Dispatches top-level subcommands (`resolver`, `rule`, `stats`, `health`)
  - Formats API responses into human-readable tabular output

- **`client.rs`** — HTTP API client
  - `GoblinClient` struct wrapping `reqwest::Client`
  - All REST calls go to the backend under `/api/v1/`
  - Generic response parser handles the `{"success": bool, "data": T}` envelope

- **`models.rs`** — Data structures
  - Serializes/deserializes `Resolver`, `Rule`, `StatEntry`, and `StatsSummary` structs
  - Maps directly to backend API payloads

### Key details

| Aspect | Detail |
|---|---|
| Runtime | Tokio async |
| HTTP client | reqwest (async) |
| Argument parsing | clap with derive macros |
| Serialization | serde (JSON) |
| Default endpoint | `http://localhost:8080` |
| Authentication | None (unauthenticated) |

## Installation

```bash
cargo build --release
cp target/release/goblin-cli /usr/local/bin/
```

## Usage

The default base URL is `http://localhost:8080`. Override it with `-b` or `--base-url`:

```bash
# Connect via frontend reverse proxy instead of directly
goblin-cli -b http://localhost:3000 health
```

### Health check

```bash
goblin-cli health
```

### Resolver management

Upstream DNS resolvers are first-class entities that the redirector can use for resolution.

```bash
# List all resolvers
goblin-cli resolver list

# Get details for a specific resolver
goblin-cli resolver get 5

# Create a new resolver
goblin-cli resolver create "8.8.8.8" "google-dns"

# Update a resolver (patch — only provided fields change)
goblin-cli resolver update 5 --ip "1.1.1.1" --domain "cloudflare-dns"

# Delete a resolver
goblin-cli resolver delete 5
```

### Rule management

DNS redirect rules map a domain to a target IP, optionally scoped to a client IP or CIDR range.

```bash
# List all rules
goblin-cli rule list

# Get a specific rule
goblin-cli rule get 1

# Create a rule (active by default)
goblin-cli rule create "evil.example.com" "127.0.0.1" \
  --client-filter "192.168.1.0/24"

# Create a rule without a client filter (applies to everyone)
goblin-cli rule create "block.example.com" "10.0.0.1" --active false

# Update individual fields (patch)
goblin-cli rule update 3 --target-ip "10.0.0.2"

# Clear the client filter entirely
goblin-cli rule update 3 --clear-client-filter

# Toggle active/inactive status
goblin-cli rule toggle 3

# Delete a rule
goblin-cli rule delete 3
```

### Statistics

The backend tracks DNS queries that match redirect rules.

```bash
# List recent stats (paginated)
goblin-cli stats list --limit 50 --offset 0

# Stats for a specific rule
goblin-cli stats for-rule 3

# Aggregate summary
goblin-cli stats summary
```

## Command reference

```
goblin-cli [OPTIONS] <COMMAND>

Options:
  -b, --base-url <BASE_URL>   Base URL of the Goblin API [default: http://localhost:8080]

Commands:
  health                       Health check against the API
  resolver                     Manage upstream DNS resolvers
    list                       List all resolvers
    get <ID>                   Get a resolver by ID
    create <IP> <DOMAIN>       Create a resolver
    update <ID> [OPTIONS]      Update a resolver
      --ip <IP>
      --domain <DOMAIN>
    delete <ID>                Delete a resolver by ID
  rule                         Manage DNS redirect rules
    list                       List all rules
    get <ID>                   Get a rule by ID
    create <DOMAIN> <TARGET_IP> [OPTIONS]
      --client-filter <FILTER>
      --active <ACTIVE>
    update <ID> [OPTIONS]      Update a rule
      --domain <DOMAIN>
      --target-ip <TARGET_IP>
      --client-filter <FILTER>
      --clear-client-filter
      --active <ACTIVE>
    toggle <ID>                Toggle active status
    delete <ID>                Delete a rule
  stats                        Query DNS statistics
    list [OPTIONS]             List stats entries
      --limit <LIMIT>
      --offset <OFFSET>
    for-rule <RULE_ID>         Stats for a specific rule
    summary                    Aggregate statistics
```
