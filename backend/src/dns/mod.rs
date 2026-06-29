pub mod resolver;
pub mod server;
pub mod handler;
pub mod cache;

pub use server::DnsServer;
pub use handler::DnsHandler;
pub use cache::{ResolverCache, ResolverCacheHandle};
pub use resolver::DnsResolver;
