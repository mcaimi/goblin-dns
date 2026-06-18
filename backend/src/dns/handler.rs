use hickory_server::authority::MessageResponseBuilder;
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_proto::op::{Header, OpCode, ResponseCode};
use hickory_proto::rr::{Record, RData, RecordType, Name};
use std::net::IpAddr;
use std::sync::Arc;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

use super::resolver::DnsResolver;
use crate::rules::{find_matching_rule, RuleCacheHandle};
use crate::stats::{StatEvent, StatsHandle};

pub struct DnsHandler {
    resolver: Arc<DnsResolver>,
    rule_cache: Option<RuleCacheHandle>,
    stats_handle: Option<StatsHandle>,
}

impl DnsHandler {
    pub fn new(resolver: Arc<DnsResolver>) -> Self {
        Self {
            resolver,
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

    async fn handle_query(&self, request: &Request) -> Result<Vec<Record>, ResponseCode> {
        let query = request.query();
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

        // Check for priority rule match first
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
                    let original_ip = match self.resolver.resolve(&domain, record_type).await {
                        Ok(ips) if !ips.is_empty() => Some(ips[0].to_string()),
                        _ => None,
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

        // No rule matched - perform normal DNS resolution
        match self.resolver.resolve(&domain, record_type).await {
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
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let query = request.query();

        // Check if this is a query
        if request.op_code() != OpCode::Query {
            let mut header = Header::response_from_request(request.header());
            header.set_response_code(ResponseCode::NotImp);

            let response = MessageResponseBuilder::from_message_request(request)
                .build_no_records(header);

            return response_handle.send_response(response).await.unwrap_or_else(|e| {
                error!("Failed to send response: {}", e);
                Header::default().into()
            });
        }

        // Handle the query
        match self.handle_query(request).await {
            Ok(records) => {
                let mut header = Header::response_from_request(request.header());
                header.set_response_code(ResponseCode::NoError);
                header.set_authoritative(false);

                let response = MessageResponseBuilder::from_message_request(request)
                    .build(header, records.iter(), &[], &[], &[]);

                response_handle.send_response(response).await.unwrap_or_else(|e| {
                    error!("Failed to send response: {}", e);
                    Header::default().into()
                })
            }
            Err(response_code) => {
                let mut header = Header::response_from_request(request.header());
                header.set_response_code(response_code);

                let response = MessageResponseBuilder::from_message_request(request)
                    .build_no_records(header);

                response_handle.send_response(response).await.unwrap_or_else(|e| {
                    error!("Failed to send response: {}", e);
                    Header::default().into()
                })
            }
        }
    }
}
