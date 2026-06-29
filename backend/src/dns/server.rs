use hickory_server::server::Server;
use hickory_net::udp::UdpSocket;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use anyhow::Result;
use tracing::info;

use crate::dns::DnsHandler;

pub struct DnsServer {
    bind_address: SocketAddr,
    tcp_enabled: bool,
    handler: DnsHandler,
}

impl DnsServer {
    pub fn new(bind_address: SocketAddr, tcp_enabled: bool, handler: DnsHandler) -> Self {
        Self {
            bind_address,
            tcp_enabled,
            handler,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting DNS server on {}", self.bind_address);

        let mut server = Server::new(self.handler);

        // Bind UDP socket
        let udp_socket = UdpSocket::bind(self.bind_address).await?;
        info!("DNS UDP server listening on {}", self.bind_address);
        server.register_socket(udp_socket);

        // Optionally bind TCP socket
        if self.tcp_enabled {
            let tcp_listener = TcpListener::bind(self.bind_address).await?;
            info!("DNS TCP server listening on {}", self.bind_address);
            // register_listener(listener, timeout, max_connections)
            server.register_listener(tcp_listener, std::time::Duration::from_secs(30), 10);
        }

        // Block forever to keep server running
        server.block_until_done().await?;

        Ok(())
    }
}
