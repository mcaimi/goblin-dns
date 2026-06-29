// hickory imports
// DNS Server
use hickory_server::server::{
    Request,
    RequestHandler,
    ResponseHandler,
    ResponseInfo
};
// zone handler for responses
use hickory_server::zone_handler::MessageResponseBuilder;
// protocol specs
use hickory_proto::{
    op::{
        Metadata,
        MessageType,
        OpCode,
        ResponseCode
    },
    rr::{
        Record,
        RData,
        RecordType,
        Name
    }
};
use hickory_net::runtime::Time;

use std::net::IpAddr;
use std::sync::Arc;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

// local imports
use crate::dns::DnsResolver;
use crate::dns::ResolverCacheHandle;
use crate::rules::{find_matching_rule, RuleCacheHandle};
use crate::stats::{StatEvent, StatsHandle};

pub struct DnsHandler {
    resolver_cache: Option<ResolverCacheHandle>,
    fallback_resolver: Option<Arc<DnsResolver>>,
    rule_cache: Option<RuleCacheHandle>,
    stats_handle: Option<StatsHandle>,
}

impl DnsHandler {
    /// Create a DNS handler that uses the resolver cache
    pub fn with_resolver_cache(resolver_cache: ResolverCacheHandle) -> Self {
        Self {
            resolver_cache: Some(resolver_cache),
            fallback_resolver: None,
            rule_cache: None,
            stats_handle: None,
        }
    }

    pub fn with_rules(mut self, rule_cache: RuleCacheHandle) -> Self {
        self.rule_cache = Some(rule_cache);
        self
    }

    pub fn with_stats(mut self, stats_handle: StatsHandle) -> Self {
        self.stats_handle = Some(stats_handle);
        self
    }

    /// Get the active resolver (from cache or fallback)
    async fn get_resolver(&self) -> Option<Arc<DnsResolver>> {
        if let Some(ref cache) = self.resolver_cache {
            cache.get_resolver().await
        } else {
            self.fallback_resolver.clone()
        }
    }

    async fn handle_query(&self, request: &Request) -> Result<Vec<Record>, ResponseCode> {
        // Get the first query (DNS requests typically have one query)
        let query = request.queries.queries().get(0)
            .ok_or(ResponseCode::FormErr)?;
        let name = query.name();
        let record_type = query.query_type();
        let client_addr = request.src().ip();

        debug!(
            "Received query for {} (type: {:?}) from {}",
            name,
            record_type,
            client_addr
        );

        // Convert name to string
        let domain = name.to_string();

        // Only handle A and AAAA records
        if !matches!(record_type, RecordType::A | RecordType::AAAA) {
            warn!("Unsupported query type: {:?}", record_type);
            return Err(ResponseCode::NotImp);
        }

        // TIER 1: Check for priority rule match first
        if let Some(ref rule_cache) = self.rule_cache {
            let rules = rule_cache.get_rules().await;

            if let Some(matched_rule) = find_matching_rule(&rules, &domain, &client_addr) {
                info!(
                    "Priority rule {} matched for {} from {} - overriding to {}",
                    matched_rule.id, domain, client_addr, matched_rule.target_ip
                );

                // Parse the target IP from the rule
                if let Ok(target_ip) = IpAddr::from_str(&matched_rule.target_ip) {
                    // Get the original IP for statistics (if available)
                    let resolver = self.get_resolver().await;
                    let original_ip = if let Some(ref resolver) = resolver {
                        match resolver.resolve(&domain, record_type).await {
                            Ok(ips) if !ips.is_empty() => Some(ips[0].to_string()),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    // Record statistics
                    if let Some(ref stats) = self.stats_handle {
                        let event = StatEvent {
                            rule_id: matched_rule.id,
                            client_ip: client_addr.to_string(),
                            requested_domain: domain.clone(),
                            original_ip,
                            overridden_ip: matched_rule.target_ip.clone(),
                        };
                        stats.try_record(event);
                    }

                    // Return the overridden IP
                    let rdata = match target_ip {
                        IpAddr::V4(ipv4) => RData::A(ipv4.into()),
                        IpAddr::V6(ipv6) => RData::AAAA(ipv6.into()),
                    };

                    let record = Record::from_rdata(Name::from(name.clone()), 300, rdata);
                    return Ok(vec![record]);
                } else {
                    error!(
                        "Invalid target IP in rule {}: {}",
                        matched_rule.id, matched_rule.target_ip
                    );
                }
            }
        }

        // TIER 2 & 3: No priority rule matched - perform normal DNS resolution
        // The resolver will automatically select:
        // - TIER 2: Domain-specific resolver if one matches
        // - TIER 3: Core resolver as fallback
        let resolver = self.get_resolver().await
            .ok_or_else(|| {
                error!("No DNS resolver available");
                ResponseCode::ServFail
            })?;

        match resolver.resolve(&domain, record_type).await {
            Ok(ips) => {
                if ips.is_empty() {
                    warn!("No results found for {}", domain);
                    return Err(ResponseCode::NXDomain);
                }

                let records: Vec<Record> = ips
                    .into_iter()
                    .map(|ip| {
                        let rdata = match ip {
                            IpAddr::V4(ipv4) => RData::A(ipv4.into()),
                            IpAddr::V6(ipv6) => RData::AAAA(ipv6.into()),
                        };

                        Record::from_rdata(Name::from(name.clone()), 300, rdata)
                    })
                    .collect();

                info!("Resolved {} to {} record(s)", domain, records.len());
                Ok(records)
            }
            Err(e) => {
                error!("Resolution failed for {}: {}", domain, e);
                Err(ResponseCode::ServFail)
            }
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for DnsHandler {
    async fn handle_request<R: ResponseHandler, T: Time>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        // Check if this is a query
        if request.metadata.op_code != OpCode::Query {
            let mut metadata = Metadata::new(
                request.metadata.id,
                MessageType::Response,
                request.metadata.op_code,
            );
            metadata.response_code = ResponseCode::NotImp;

            let response = MessageResponseBuilder::from_message_request(request)
                .build_no_records(metadata);

            return response_handle.send_response(response)
                .await
                .unwrap_or_else(|_| {
                    let mut meta = Metadata::new(
                        request.metadata.id,
                        MessageType::Response,
                        OpCode::Query,
                    );
                    meta.response_code = ResponseCode::ServFail;
                    ResponseInfo::from(hickory_proto::op::Header {
                        metadata: meta,
                        counts: hickory_proto::op::HeaderCounts::default(),
                    })
                });
        }

        // Handle the query
        match self.handle_query(request).await {
            Ok(records) => {
                let mut metadata = Metadata::new(
                    request.metadata.id,
                    MessageType::Response,
                    OpCode::Query,
                );
                metadata.response_code = ResponseCode::NoError;
                metadata.authoritative = false;

                let response = MessageResponseBuilder::from_message_request(request)
                    .build(metadata, &records, &[], &[], &[]);

                response_handle.send_response(response)
                    .await
                    .unwrap_or_else(|_| {
                        let mut meta = Metadata::new(
                            request.metadata.id,
                            MessageType::Response,
                            OpCode::Query,
                        );
                        meta.response_code = ResponseCode::ServFail;
                        ResponseInfo::from(hickory_proto::op::Header {
                            metadata: meta,
                            counts: hickory_proto::op::HeaderCounts::default(),
                        })
                    })
            }
            Err(response_code) => {
                let mut metadata = Metadata::new(
                    request.metadata.id,
                    MessageType::Response,
                    OpCode::Query,
                );
                metadata.response_code = response_code;

                let response = MessageResponseBuilder::from_message_request(request)
                    .build_no_records(metadata);

                response_handle.send_response(response)
                    .await
                    .unwrap_or_else(|_| {
                        let mut meta = Metadata::new(
                            request.metadata.id,
                            MessageType::Response,
                            OpCode::Query,
                        );
                        meta.response_code = ResponseCode::ServFail;
                        ResponseInfo::from(hickory_proto::op::Header {
                            metadata: meta,
                            counts: hickory_proto::op::HeaderCounts::default(),
                        })
                    })
            }
        }
    }
}
