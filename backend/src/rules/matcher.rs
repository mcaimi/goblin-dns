use crate::db::models::PriorityRule;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::str::FromStr;
use tracing::{debug, warn};

/// Matches a client IP against a priority rule's client filter
/// Returns true if the rule applies to this client
pub fn client_matches(rule: &PriorityRule, client_ip: &IpAddr) -> bool {
    match &rule.client_filter {
        // No filter means rule applies to all clients
        None => {
            debug!("Rule {} has no client filter - matches all clients", rule.id);
            true
        }
        Some(filter) => {
            // Try to parse as exact IP address first
            if let Ok(filter_ip) = IpAddr::from_str(filter) {
                let matches = &filter_ip == client_ip;
                debug!(
                    "Exact IP match: rule {} filter {} vs client {} = {}",
                    rule.id, filter_ip, client_ip, matches
                );
                return matches;
            }

            // Try to parse as CIDR network
            if let Ok(network) = IpNetwork::from_str(filter) {
                let matches = network.contains(*client_ip);
                debug!(
                    "CIDR match: rule {} filter {} vs client {} = {}",
                    rule.id, filter, client_ip, matches
                );
                return matches;
            }

            // Invalid filter format
            warn!(
                "Rule {} has invalid client filter '{}' - skipping",
                rule.id, filter
            );
            false
        }
    }
}

/// Matches a domain against a priority rule's domain pattern
/// Currently supports exact matching only
/// Future: Add wildcard support (*.example.com)
pub fn domain_matches(rule: &PriorityRule, requested_domain: &str) -> bool {
    // Normalize both domains to lowercase for case-insensitive comparison
    let rule_domain = rule.domain.to_lowercase();
    let requested = requested_domain.to_lowercase();

    // Remove trailing dots from domains (DNS normalization)
    let rule_domain = rule_domain.trim_end_matches('.');
    let requested = requested.trim_end_matches('.');

    let matches = rule_domain == requested;

    debug!(
        "Domain match: rule {} domain '{}' vs requested '{}' = {}",
        rule.id, rule_domain, requested, matches
    );

    matches
}

/// Finds the best matching priority rule for a given domain and client IP
/// Returns the rule with the most specific client filter that matches
/// Priority: Exact IP > CIDR subnet > No filter (all clients)
pub fn find_matching_rule(
    rules: &[PriorityRule],
    domain: &str,
    client_ip: &IpAddr,
) -> Option<PriorityRule> {
    debug!(
        "Finding matching rule for domain '{}' from client {}",
        domain, client_ip
    );

    // Filter to only active rules that match the domain
    let mut matching_rules: Vec<&PriorityRule> = rules
        .iter()
        .filter(|r| r.active && domain_matches(r, domain))
        .collect();

    if matching_rules.is_empty() {
        debug!("No matching rules found for domain '{}'", domain);
        return None;
    }

    debug!("Found {} potential rule(s) for domain '{}'", matching_rules.len(), domain);

    // Further filter by client IP
    matching_rules.retain(|r| client_matches(r, client_ip));

    if matching_rules.is_empty() {
        debug!("No rules matched client IP {}", client_ip);
        return None;
    }

    debug!("Found {} matching rule(s) after client filter", matching_rules.len());

    // Sort by specificity: exact IP > CIDR > no filter
    matching_rules.sort_by(|a, b| {
        match (&a.client_filter, &b.client_filter) {
            // Both have filters - compare specificity
            (Some(filter_a), Some(filter_b)) => {
                // Exact IP vs CIDR
                let a_is_exact = IpAddr::from_str(filter_a).is_ok();
                let b_is_exact = IpAddr::from_str(filter_b).is_ok();

                match (a_is_exact, b_is_exact) {
                    (true, false) => std::cmp::Ordering::Less,    // a is more specific
                    (false, true) => std::cmp::Ordering::Greater, // b is more specific
                    _ => {
                        // Both same type - compare CIDR prefix length if both are networks
                        if let (Ok(net_a), Ok(net_b)) = (
                            IpNetwork::from_str(filter_a),
                            IpNetwork::from_str(filter_b),
                        ) {
                            // Larger prefix = more specific
                            net_b.prefix().cmp(&net_a.prefix())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                }
            }
            // a has filter, b doesn't - a is more specific
            (Some(_), None) => std::cmp::Ordering::Less,
            // b has filter, a doesn't - b is more specific
            (None, Some(_)) => std::cmp::Ordering::Greater,
            // Neither has filter - equal
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    // Return the most specific rule
    let best_match = matching_rules.first().map(|&r| r.clone());

    if let Some(ref rule) = best_match {
        debug!(
            "Selected rule {} (domain: '{}', client_filter: {:?}, target: {})",
            rule.id, rule.domain, rule.client_filter, rule.target_ip
        );
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule(id: i64, domain: &str, client_filter: Option<&str>, target_ip: &str) -> PriorityRule {
        PriorityRule {
            id,
            domain: domain.to_string(),
            client_filter: client_filter.map(|s| s.to_string()),
            target_ip: target_ip.to_string(),
            active: true,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn test_client_matches_no_filter() {
        let rule = create_test_rule(1, "example.com", None, "127.0.0.1");
        let client = IpAddr::from_str("192.168.1.100").unwrap();
        assert!(client_matches(&rule, &client));
    }

    #[test]
    fn test_client_matches_exact_ip() {
        let rule = create_test_rule(1, "example.com", Some("192.168.1.100"), "127.0.0.1");
        let client = IpAddr::from_str("192.168.1.100").unwrap();
        assert!(client_matches(&rule, &client));

        let other_client = IpAddr::from_str("192.168.1.101").unwrap();
        assert!(!client_matches(&rule, &other_client));
    }

    #[test]
    fn test_client_matches_cidr() {
        let rule = create_test_rule(1, "example.com", Some("192.168.1.0/24"), "127.0.0.1");

        let client1 = IpAddr::from_str("192.168.1.100").unwrap();
        assert!(client_matches(&rule, &client1));

        let client2 = IpAddr::from_str("192.168.1.255").unwrap();
        assert!(client_matches(&rule, &client2));

        let client3 = IpAddr::from_str("192.168.2.1").unwrap();
        assert!(!client_matches(&rule, &client3));
    }

    #[test]
    fn test_domain_matches() {
        let rule = create_test_rule(1, "example.com", None, "127.0.0.1");
        assert!(domain_matches(&rule, "example.com"));
        assert!(domain_matches(&rule, "EXAMPLE.COM")); // Case insensitive
        assert!(domain_matches(&rule, "example.com.")); // Trailing dot
        assert!(!domain_matches(&rule, "sub.example.com"));
    }

    #[test]
    fn test_find_matching_rule_priority() {
        let rules = vec![
            create_test_rule(1, "example.com", None, "10.0.0.1"),
            create_test_rule(2, "example.com", Some("192.168.1.0/24"), "10.0.0.2"),
            create_test_rule(3, "example.com", Some("192.168.1.100"), "10.0.0.3"),
        ];

        let client = IpAddr::from_str("192.168.1.100").unwrap();
        let matched = find_matching_rule(&rules, "example.com", &client);

        // Should match rule 3 (exact IP is most specific)
        assert_eq!(matched.unwrap().id, 3);
    }
}
