# GOBLIN - DNS Redirector for Security Researchers

A dynamic DNS redirector with REST API and Web UI, designed for security researchers and reverse engineers.

## Features

- **Multithreaded DNS Resolver** - Built with Rust and Tokio for high performance
- **Dynamic Priority Rules** - Override DNS resolution for specific domains and clients
- **Statistics Collection** - Track DNS query patterns and rule hits
- **REST API** - Full CRUD interface for configuration
- **React Web UI** - Modern interface for managing rules and viewing statistics
- **SQLite Database** - Persistent storage for all configuration

## Project Status

- ✅ Rust project structure
- ✅ SQLite database schema and migrations
- ✅ Basic DNS resolver (UDP/TCP)
- ✅ Database layer with models and queries
- ✅ Configuration management
- ✅ Logging infrastructure
- ✅ Priority rule matching (domain + client IP/CIDR filtering)
- ✅ In-memory rule cache with periodic refresh
- ✅ Statistics collection via async channels
- ✅ Integration with DNS resolver
- ✅ Full REST API with Axum
- ✅ CRUD endpoints for resolvers and rules
- ✅ Statistics retrieval endpoints
- ✅ CORS support for frontend
- ✅ React + TypeScript web interface
- ✅ TanStack Query for state management
- ✅ Upstream resolver management UI
- ✅ Priority rule management UI
- ✅ Statistics dashboard with real-time updates

## Quick Start

### Prerequisites

- Rust 1.70 or later
- SQLite 3
- Node.js 20+ (for frontend development)
- Podman 5.8.2+ or Docker (for containerized deployment)

### Option 1: Quick Start with Podman Compose (Recommended)

```bash
# Build and start all services
podman compose -f container-compose.yml up -d

# Access the application
# - Web UI: http://localhost:3000
# - API (via frontend proxy): http://localhost:3000/api/v1
# - DNS Server: localhost:53 (UDP/TCP)

# View logs
podman compose -f container-compose.yml logs -f

# Stop services
podman compose -f container-compose.yml down
```

### Option 2: Manual Build and Run

**Backend:**

```bash
cd backend
cargo build --release

# Initialize database
sqlite3 goblin.db < migrations/001_initial.sql

# Run the server
cargo run
```

The DNS server will start on:

- UDP: `0.0.0.0:53`
- TCP: `0.0.0.0:53`

Note: Binding to port 53 requires root/administrator privileges. You can use a different port for testing:

```bash
cargo run -- --dns-bind-address 0.0.0.0:5353
```

**Frontend:**

```bash
cd frontend
npm install
npm run dev  # Development server on http://localhost:5173
```

### Configuration

Configuration can be set via:

1. Environment variables
2. Command-line arguments

#### Environment Variables

```bash
DNS_BIND_ADDRESS=0.0.0.0:53
DNS_TCP_ENABLED=true
API_BIND_ADDRESS=0.0.0.0:8080
API_CORS_ENABLED=true
DATABASE_PATH=./goblin.db
RULES_CACHE_TTL=60
LOG_LEVEL=info
DEFAULT_UPSTREAM=8.8.8.8:53
```

#### Command-Line Arguments

```bash
cargo run -- \
  --dns-bind-address 0.0.0.0:53 \
  --api-bind-address 0.0.0.0:8080 \
  --database-path ./goblin.db \
  --default-upstream 8.8.8.8:53 \
  --log-level debug
```

## Testing DNS Resolution

Once the server is running, test with `dig`:

```bash
# Test from another terminal (server must be running as root on port 53)
dig @localhost google.com

# Or if running on custom port:
dig @localhost -p 5353 google.com
```

### Testing Priority Rules (Phase 2)

Add a test priority rule to the database:

```bash
# Insert a rule that redirects evil.com to localhost for all clients
sqlite3 goblin.db "INSERT INTO priority_rules (domain, client_filter, target_ip, active, created_at, updated_at) VALUES ('evil.com', NULL, '127.0.0.1', 1, $(date +%s), $(date +%s));"

# Test the rule
dig @localhost -p 5353 evil.com  # Should return 127.0.0.1

# Check statistics
sqlite3 goblin.db "SELECT * FROM statistics;"
```

### Testing REST API

**Architecture Note:** The frontend now acts as a reverse proxy for the backend API. All API requests should be sent through the frontend at `http://localhost:3000/api/v1`. The backend API port (8080) is not directly exposed in production deployments for security reasons.

**API Endpoints:**

```bash
# Health check (via frontend proxy)
curl http://localhost:3000/api/v1/health

# Or direct backend access (development only, if port 8080 is exposed)
curl http://localhost:8080/api/v1/health

# Upstream Resolvers
GET    /api/v1/resolvers              # List all
GET    /api/v1/resolvers/:id          # Get one
POST   /api/v1/resolvers              # Create
PUT    /api/v1/resolvers/:id          # Update
DELETE /api/v1/resolvers/:id          # Delete

# Priority Rules
GET    /api/v1/rules                  # List all
GET    /api/v1/rules/:id              # Get one
POST   /api/v1/rules                  # Create
PUT    /api/v1/rules/:id              # Update
PATCH  /api/v1/rules/:id/toggle       # Toggle active/inactive
DELETE /api/v1/rules/:id              # Delete

# Statistics
GET    /api/v1/stats                  # Get all (paginated)
GET    /api/v1/stats/rules/:rule_id   # Get for specific rule
GET    /api/v1/stats/summary          # Get summary
```

**Example API Usage:**

```bash
# Create a priority rule (via frontend proxy)
curl -X POST http://localhost:3000/api/v1/rules \
  -H "Content-Type: application/json" \
  -d '{
    "domain": "malicious.com",
    "client_filter": "192.168.1.0/24",
    "target_ip": "127.0.0.1",
    "active": true
  }'

# List all rules
curl http://localhost:3000/api/v1/rules | jq '.'

# Toggle a rule
curl -X PATCH http://localhost:3000/api/v1/rules/1/toggle

# Get statistics summary
curl http://localhost:3000/api/v1/stats/summary | jq '.'
```

## Architecture

### System Design

GOBLIN uses a **frontend-as-proxy** architecture for security and simplified deployment:

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       │ HTTP (Port 3000)
       ▼
┌─────────────────────────┐
│  Frontend (nginx)       │
│  - Serves React UI      │
│  - Proxies /api/* to    │
│    backend              │
└──────┬──────────────────┘
       │
       │ Internal network only
       ▼
┌─────────────────────────┐
│  Backend (Rust)         │
│  - DNS Server (Port 53) │
│  - REST API (Port 8080) │
│    (not exposed)        │
└─────────────────────────┘
```

**Security Benefits:**
- Backend API port is not exposed to users
- Single entry point (port 3000) for all HTTP traffic
- nginx handles SSL termination and security headers
- Reduced attack surface

### Backend (Rust)

```
backend/
├── src/
│   ├── main.rs           # Application entry point with signal handling
│   ├── config.rs         # Configuration management
│   ├── db/               # Database layer
│   │   ├── models.rs     # Data models
│   │   └── queries.rs    # Database queries
│   ├── dns/              # DNS resolution
│   │   ├── server.rs     # DNS server (UDP/TCP)
│   │   ├── handler.rs    # Request handler
│   │   └── resolver.rs   # Core resolution logic
│   ├── rules/            # Rule engine
│   │   ├── matcher.rs    # Domain + client IP matching
│   │   └── cache.rs      # In-memory rule cache
│   ├── stats/            # Statistics collection
│   │   └── collector.rs  # Async stats collector
│   └── api/              # REST API with Axum
└── migrations/
    └── 001_initial.sql   # Database schema
```

### Frontend (React + TypeScript)

```
frontend/
├── src/
│   ├── api/              # API client (uses relative paths)
│   ├── components/       # React components
│   ├── hooks/            # Custom React hooks
│   └── types/            # TypeScript type definitions
├── nginx.conf            # nginx config with API proxy
└── docker-entrypoint.sh  # Container startup script
```

### Database Schema

**upstream_resolvers** - DNS servers to forward queries to

- Maps domains to specific upstream resolvers
- "core" domain indicates fallback resolver

**priority_rules** - Override DNS resolution

- Domain to IP address mapping
- Optional client IP/subnet filtering
- Active/inactive status

**statistics** - Track rule hits

- Query timestamps and client IPs
- Original vs overridden IP addresses
- Linked to priority rules

## Use Case Examples

1. **Content Filtering** - Redirect malicious URLs to localhost or a courtesy page
2. **URL Interception** - Redirect traffic for protocol inspection
3. **Domain Blacklisting** - Block specific domains during investigations
4. **Research Environment** - Create isolated DNS environments for analysis

## Dependencies

### Backend
- **tokio** - Async runtime
- **hickory-dns** - DNS protocol implementation
- **axum** - Web framework for REST API
- **sqlx** - Async database driver
- **tracing** - Structured logging
- **clap** - CLI argument parsing
- **tower-http** - CORS middleware

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **TanStack Query** - Data fetching and caching
- **axios** - HTTP client
- **Tailwind CSS** - Styling
- **nginx** - Reverse proxy and static file serving

## Development

### Backend Development

**Running Tests:**

```bash
cd backend
cargo test
```

**Building for Production:**

```bash
cd backend
cargo build --release
```

The binary will be in `target/release/goblin`.

**Logging:**

Set log level via environment variable or CLI:

```bash
LOG_LEVEL=debug cargo run
```

Available levels: trace, debug, info, warn, error

### Frontend Development

**Development Server:**

```bash
cd frontend
npm install
npm run dev
```

The development server starts on `http://localhost:5173` with hot module reloading.

**Environment Configuration:**

Create a `.env.local` file for local development:

```bash
# Points to your local backend API
VITE_API_BASE_URL=http://localhost:8080/api/v1
```

**Building for Production:**

```bash
npm run build
```

The production build will be in `frontend/dist/`.

**Type Checking:**

```bash
npm run type-check
```

**Linting:**

```bash
npm run lint
```

## Deployment

### Quick Start with Podman Compose

```bash
# Start all services
podman compose up -d

# View logs
podman compose logs -f

# Stop services
podman compose down
```

Services:

- **DNS Server**: `localhost:53` (UDP/TCP)
- **Web UI**: `http://localhost:3000`
- **REST API**: `http://localhost:3000/api/v1` (proxied through frontend)

### Production Deployment

See [DEPLOY.md](DEPLOY.md) for comprehensive deployment instructions.

## Documentation

- [README.md](README.md) - This file, project overview
- [DEPLOY.md](DEPLOY.md) - Deployment and operations guide

## License

See LICENSE file for details.

## Acknowledgments

Built with:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Hickory DNS](https://github.com/hickory-dns/hickory-dns) - DNS protocol implementation
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [React](https://react.dev/) - UI framework
- [TanStack Query](https://tanstack.com/query) - Data fetching library
- [OpenCode](https://opencode.ai) - AI Coding Assistant (Qwen 3.6 Model)
