use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol};
use hickory_proto::rr::{Name, RecordType};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use anyhow::{Result, anyhow};
use tracing::{debug, info, warn};

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    pub async fn new(upstream_addr: &str) -> Result<Self> {
        // Parse upstream address
        let upstream_socket: SocketAddr = upstream_addr.parse()
            .map_err(|_| anyhow!("Invalid upstream address: {}", upstream_addr))?;

        // Create custom resolver config with the upstream server
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: upstream_socket,
            protocol: Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: true,
            bind_addr: None,
        });

        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());

        info!("DNS resolver initialized with upstream: {}", upstream_addr);

        Ok(Self { resolver })
    }

    pub async fn resolve(&self, domain: &str, record_type: RecordType) -> Result<Vec<IpAddr>> {
        debug!("Resolving domain: {} (type: {:?})", domain, record_type);

        let name = Name::from_str(domain)
            .map_err(|e| anyhow!("Invalid domain name '{}': {}", domain, e))?;

        match record_type {
            RecordType::A => {
                let response = self.resolver.ipv4_lookup(name).await
                    .map_err(|e| anyhow!("A record lookup failed for '{}': {}", domain, e))?;

                let ips: Vec<IpAddr> = response.iter().map(|ip| IpAddr::V4(ip.0)).collect();
                debug!("Resolved {} to {:?}", domain, ips);
                Ok(ips)
            }
            RecordType::AAAA => {
                let response = self.resolver.ipv6_lookup(name).await
                    .map_err(|e| anyhow!("AAAA record lookup failed for '{}': {}", domain, e))?;

                let ips: Vec<IpAddr> = response.iter().map(|ip| IpAddr::V6(ip.0)).collect();
                debug!("Resolved {} to {:?}", domain, ips);
                Ok(ips)
            }
            _ => {
                warn!("Unsupported record type: {:?}", record_type);
                Ok(vec![])
            }
        }
    }

    pub async fn resolve_a(&self, domain: &str) -> Result<Vec<IpAddr>> {
        self.resolve(domain, RecordType::A).await
    }

    pub async fn resolve_aaaa(&self, domain: &str) -> Result<Vec<IpAddr>> {
        self.resolve(domain, RecordType::AAAA).await
    }
}
