/// Test to verify the three-tier DNS resolution flow
///
/// Resolution flow:
/// 1. Check priority rules - if match, use rule override
/// 2. Check domain-specific upstream resolvers - if domain matches, use that resolver
/// 3. Fall back to "core" upstream resolvers for all other queries

#[cfg(test)]
mod tests {
    use goblin::dns::resolver::{DnsResolver};
    use goblin::db::models::UpstreamResolver;

    #[tokio::test]
    async fn test_domain_specific_resolver_selection() {
        // Create test upstream resolvers
        let upstreams = vec![
            UpstreamResolver {
                id: 1,
                ip_address: "8.8.8.8:53".to_string(),
                domain: "core".to_string(),
                created_at: 0,
                updated_at: 0,
            },
            UpstreamResolver {
                id: 2,
                ip_address: "1.1.1.1:53".to_string(),
                domain: "example.com".to_string(),
                created_at: 0,
                updated_at: 0,
            },
            UpstreamResolver {
                id: 3,
                ip_address: "9.9.9.9:53".to_string(),
                domain: "test.org".to_string(),
                created_at: 0,
                updated_at: 0,
            },
        ];

        // Create resolver from upstreams
        let resolver = DnsResolver::from_upstreams(upstreams).await.unwrap();

        // Test 1: Domain with specific resolver (example.com)
        // Should use resolver at 1.1.1.1:53
        let test1 = resolver.select_resolver("example.com");
        assert!(!test1.is_none());
        assert!(test1.unwrap().domain == "example.com");
        assert!(test1.unwrap().ip_address == "1.1.1.1:53");

        // Test 2: Subdomain of domain with specific resolver (sub.example.com)
        // Should use resolver at 1.1.1.1:53
        let test2 = resolver.select_resolver("sub.example.com");
        assert!(!test2.is_none());
        assert!(test2.unwrap().domain == "example.com");
        assert!(test2.unwrap().ip_address == "1.1.1.1:53");

        // Test 3: Domain without specific resolver (google.com)
        // Should use core resolver at 8.8.8.8:53
        let test3 = resolver.select_resolver("google.com");
        assert!(!test3.is_none());
        assert!(test3.unwrap().domain == "core");
        assert!(test3.unwrap().ip_address == "8.8.8.8:53");

        // Test 4: Another domain with specific resolver (test.org)
        // Should use resolver at 9.9.9.9:53
        let test4 = resolver.select_resolver("test.org");
        assert!(!test4.is_none());
        assert!(test4.unwrap().domain == "test.org");
        assert!(test4.unwrap().ip_address == "9.9.9.9:53");
    }

    #[tokio::test]
    async fn test_core_resolver_fallback() {
        // Create test upstream with only core resolvers
        let upstreams = vec![
            UpstreamResolver {
                id: 1,
                ip_address: "8.8.8.8:53".to_string(),
                domain: "core".to_string(),
                created_at: 0,
                updated_at: 0,
            },
        ];

        let resolver = DnsResolver::from_upstreams(upstreams).await.unwrap();

        // All queries should use the core resolver
        // This test verifies that without domain-specific resolvers,
        // everything falls back to core
        let test1 = resolver.select_resolver("google.com");
        let test2 = resolver.select_resolver("www.redhat.com");
        let test3 = resolver.select_resolver("wikipedia.org");
        assert!(test1.unwrap().domain == "core");
        assert!(test2.unwrap().domain == "core");
        assert!(test3.unwrap().domain == "core");
    }

    #[tokio::test]
    async fn test_subdomain_matching() {
        // Test that subdomains correctly match their parent domain resolver
        let upstreams = vec![
            UpstreamResolver {
                id: 1,
                ip_address: "8.8.8.8:53".to_string(),
                domain: "core".to_string(),
                created_at: 0,
                updated_at: 0,
            },
            UpstreamResolver {
                id: 2,
                ip_address: "1.1.1.1:53".to_string(),
                domain: "example.com".to_string(),
                created_at: 0,
                updated_at: 0,
            },
        ];

        let resolver = DnsResolver::from_upstreams(upstreams).await.unwrap();

        // These should all match the example.com resolver:
        // - example.com
        // - www.example.com
        // - api.example.com
        // - deep.nested.example.com
        let test1_domains = vec!["example.com", "www.example.com", "api.example.com", "deep.nested.example.com"];
        for dom in test1_domains.iter() {
            let choice = resolver
                .select_resolver(dom)
                .unwrap();
            assert!(choice.domain == "example.com");
            assert!(choice.ip_address == "1.1.1.1:53");
        }
        
        // These should NOT match and use core:
        // - notexample.com
        // - example.org
        let test2_domains = vec!["notexample.com", "example.org"];
        for dom in test2_domains.iter() {
            let choice = resolver
                .select_resolver(dom)
                .unwrap();
            assert!(choice.domain == "core");
            assert!(choice.ip_address == "8.8.8.8:53");
        }

    }
}
