-- Upstream DNS resolvers
CREATE TABLE IF NOT EXISTS upstream_resolvers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_address TEXT NOT NULL,           -- IP of the upstream DNS server
    domain TEXT NOT NULL,                -- Domain pattern or "core" for fallback
    created_at INTEGER NOT NULL,         -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(ip_address, domain)
);

CREATE INDEX IF NOT EXISTS idx_upstream_domain ON upstream_resolvers(domain);

-- Priority rules for DNS resolution override
CREATE TABLE IF NOT EXISTS priority_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT NOT NULL,                -- Domain to override (e.g., "google.com")
    client_filter TEXT,                  -- Client IP/subnet (NULL = all clients)
    target_ip TEXT NOT NULL,             -- IP to return instead of actual resolution
    active INTEGER NOT NULL DEFAULT 1,   -- 1 = active, 0 = inactive
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(domain, client_filter)
);

CREATE INDEX IF NOT EXISTS idx_priority_domain ON priority_rules(domain);
CREATE INDEX IF NOT EXISTS idx_priority_active ON priority_rules(active);

-- Statistics for priority rule hits
CREATE TABLE IF NOT EXISTS statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,            -- Foreign key to priority_rules
    client_ip TEXT NOT NULL,             -- Client that made the request
    requested_domain TEXT NOT NULL,      -- Domain that was requested
    original_ip TEXT,                    -- IP from actual DNS resolution (if queried)
    overridden_ip TEXT NOT NULL,         -- IP returned to client (from rule)
    timestamp INTEGER NOT NULL,          -- Unix timestamp of the hit
    FOREIGN KEY (rule_id) REFERENCES priority_rules(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_stats_rule_id ON statistics(rule_id);
CREATE INDEX IF NOT EXISTS idx_stats_timestamp ON statistics(timestamp);
CREATE INDEX IF NOT EXISTS idx_stats_client_ip ON statistics(client_ip);
