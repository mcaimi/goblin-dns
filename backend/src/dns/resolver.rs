use hickory_resolver::TokioResolver;
use hickory_resolver::config::{ResolverConfig, NameServerConfig, ConnectionConfig};
use hickory_proto::rr::{Name, RecordType};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{debug, info, warn};

use crate::db::models::UpstreamResolver;

/// A DNS resolver that can be associated with a specific domain or marked as "core"
#[derive(Clone)]
pub struct DomainResolver {
    pub resolver: TokioResolver,
    pub domain: String,
    pub ip_address: String,
}

pub struct DnsResolver {
    // Map of domain patterns to their specific resolvers
    domain_resolvers: HashMap<String, DomainResolver>,
    // Core resolvers that handle all non-matched queries
    core_resolvers: Vec<DomainResolver>,
}

impl DnsResolver {
    /// Create a new DNS resolver from a list of upstream resolvers
    pub async fn from_upstreams(upstreams: Vec<UpstreamResolver>) -> Result<Self> {
        let mut domain_resolvers = HashMap::new();
        let mut core_resolvers = Vec::new();

        for upstream in upstreams {
            let resolver = Self::create_resolver(&upstream.ip_address)?;

            let domain_resolver = DomainResolver {
                resolver,
                domain: upstream.domain.clone(),
                ip_address: upstream.ip_address.clone(),
            };

            if upstream.domain == "core" {
                info!("Adding core resolver: {}", domain_resolver.ip_address);
                core_resolvers.push(domain_resolver);
            } else {
                info!("Adding domain-specific resolver: {} for domain '{}'",
                      domain_resolver.ip_address, domain_resolver.domain);
                domain_resolvers.insert(upstream.domain.clone(), domain_resolver);
            }
        }

        if core_resolvers.is_empty() {
            warn!("No core resolvers configured - will fail for unmatched domains");
        }

        Ok(Self {
            domain_resolvers,
            core_resolvers,
        })
    }

    /// Helper to create a TokioResolver for a given upstream address
    fn create_resolver(upstream_addr: &str) -> Result<TokioResolver> {
        // Parse upstream address
        let upstream_socket: SocketAddr = upstream_addr.parse()
            .map_err(|_| anyhow!("Invalid upstream address: {}", upstream_addr))?;

        // Create connection config for UDP
        let mut conn_config = ConnectionConfig::udp();
        conn_config.port = upstream_socket.port();

        // Create custom resolver config with the upstream server
        let name_server = NameServerConfig::new(
            upstream_socket.ip(),
            true, // trust_negative_responses
            vec![conn_config],
        );
        let config = ResolverConfig::from_parts(
            None,
            vec![],
            vec![name_server],
        );

        use hickory_resolver::net::runtime::TokioRuntimeProvider;
        let resolver = TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
            .build()?;
        Ok(resolver)
    }

    /// Select the appropriate resolver for a given domain
    /// Returns None if no resolver is available
    fn select_resolver(&self, domain: &str) -> Option<&DomainResolver> {
        let normalized = domain.to_lowercase().trim_end_matches('.').to_string();

        // First, check for exact domain match
        if let Some(resolver) = self.domain_resolvers.get(&normalized) {
            debug!("Selected domain-specific resolver {} for domain '{}'",
                   resolver.ip_address, domain);
            return Some(resolver);
        }

        // Check for subdomain matches (e.g., "sub.example.com" matches "example.com")
        for (pattern, resolver) in &self.domain_resolvers {
            if normalized.ends_with(&format!(".{}", pattern)) || normalized == *pattern {
                debug!("Selected domain-specific resolver {} for domain '{}' (pattern: '{}')",
                       resolver.ip_address, domain, pattern);
                return Some(resolver);
            }
        }

        // Fall back to first core resolver
        if let Some(resolver) = self.core_resolvers.first() {
            debug!("Using core resolver {} for domain '{}'",
                   resolver.ip_address, domain);
            return Some(resolver);
        }

        warn!("No resolver available for domain '{}'", domain);
        None
    }

    pub async fn resolve(&self, domain: &str, record_type: RecordType) -> Result<Vec<IpAddr>> {
        debug!("Resolving domain: {} (type: {:?})", domain, record_type);

        // Select the appropriate resolver based on domain
        let domain_resolver = self.select_resolver(domain)
            .ok_or_else(|| anyhow!("No resolver available for domain '{}'", domain))?;

        let name = Name::from_str(domain)
            .map_err(|e| anyhow!("Invalid domain name '{}': {}", domain, e))?;

        match record_type {
            RecordType::A => {
                let response = domain_resolver.resolver.ipv4_lookup(name).await
                    .map_err(|e| anyhow!("A record lookup failed for '{}': {}", domain, e))?;

                let ips: Vec<IpAddr> = response.answers()
                    .iter()
                    .filter_map(|record| {
                        if let hickory_proto::rr::RData::A(ipv4) = &record.data {
                            Some(IpAddr::V4((*ipv4).into()))
                        } else {
                            None
                        }
                    })
                    .collect();
                debug!("Resolved {} to {:?} using {}", domain, ips, domain_resolver.ip_address);
                Ok(ips)
            }
            RecordType::AAAA => {
                let response = domain_resolver.resolver.ipv6_lookup(name).await
                    .map_err(|e| anyhow!("AAAA record lookup failed for '{}': {}", domain, e))?;

                let ips: Vec<IpAddr> = response.answers()
                    .iter()
                    .filter_map(|record| {
                        if let hickory_proto::rr::RData::AAAA(ipv6) = &record.data {
                            Some(IpAddr::V6((*ipv6).into()))
                        } else {
                            None
                        }
                    })
                    .collect();
                debug!("Resolved {} to {:?} using {}", domain, ips, domain_resolver.ip_address);
                Ok(ips)
            }
            _ => {
                warn!("Unsupported record type: {:?}", record_type);
                Ok(vec![])
            }
        }
    }
}
