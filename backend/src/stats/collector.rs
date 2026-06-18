use crate::db::Database;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Statistics event to be recorded
#[derive(Debug, Clone)]
pub struct StatEvent {
    pub rule_id: i64,
    pub client_ip: String,
    pub requested_domain: String,
    pub original_ip: Option<String>,
    pub overridden_ip: String,
}

/// Statistics collector that runs in the background
/// Receives statistics events via channel and writes them to database
pub struct StatsCollector {
    db: Arc<Database>,
    receiver: mpsc::Receiver<StatEvent>,
}

impl StatsCollector {
    /// Creates a new statistics collector
    /// Returns the collector and a sender channel for submitting events
    pub fn new(db: Arc<Database>, buffer_size: usize) -> (Self, mpsc::Sender<StatEvent>) {
        let (sender, receiver) = mpsc::channel(buffer_size);

        let collector = Self { db, receiver };

        (collector, sender)
    }

    /// Runs the statistics collector in the background
    /// This should be spawned as a Tokio task
    pub async fn run(mut self) {
        info!("Statistics collector started");

        while let Some(event) = self.receiver.recv().await {
            debug!(
                "Recording stat for rule {} - domain '{}' from client {}",
                event.rule_id, event.requested_domain, event.client_ip
            );

            if let Err(e) = self
                .db
                .record_stat(
                    event.rule_id,
                    &event.client_ip,
                    &event.requested_domain,
                    event.original_ip.as_deref(),
                    &event.overridden_ip,
                )
                .await
            {
                error!("Failed to record statistics: {}", e);
            }
        }

        warn!("Statistics collector channel closed - shutting down");
    }
}

/// Handle for submitting statistics events
/// This is cloneable and can be shared across tasks
#[derive(Clone)]
pub struct StatsHandle {
    sender: mpsc::Sender<StatEvent>,
}

impl StatsHandle {
    pub fn new(sender: mpsc::Sender<StatEvent>) -> Self {
        Self { sender }
    }

    /// Records a statistics event (non-blocking)
    /// Returns true if the event was queued successfully
    pub async fn record(&self, event: StatEvent) -> bool {
        match self.sender.send(event).await {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to send statistics event: {}", e);
                false
            }
        }
    }

    /// Tries to record a statistics event without blocking
    /// Returns true if the event was queued successfully
    pub fn try_record(&self, event: StatEvent) -> bool {
        match self.sender.try_send(event) {
            Ok(_) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("Statistics channel full - dropping event");
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("Statistics channel closed");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stats_handle() {
        let (sender, mut receiver) = mpsc::channel(10);
        let handle = StatsHandle::new(sender);

        let event = StatEvent {
            rule_id: 1,
            client_ip: "192.168.1.100".to_string(),
            requested_domain: "example.com".to_string(),
            original_ip: Some("93.184.216.34".to_string()),
            overridden_ip: "127.0.0.1".to_string(),
        };

        // Record event
        assert!(handle.record(event.clone()).await);

        // Verify it was received
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.rule_id, event.rule_id);
        assert_eq!(received.client_ip, event.client_ip);
    }

    #[tokio::test]
    async fn test_try_record() {
        let (sender, mut receiver) = mpsc::channel(10);
        let handle = StatsHandle::new(sender);

        let event = StatEvent {
            rule_id: 1,
            client_ip: "192.168.1.100".to_string(),
            requested_domain: "example.com".to_string(),
            original_ip: None,
            overridden_ip: "127.0.0.1".to_string(),
        };

        // First try_record should succeed
        assert!(handle.try_record(event.clone()));

        // Consume the event
        receiver.recv().await.unwrap();

        // Second should also succeed
        assert!(handle.try_record(event.clone()));
    }
}
