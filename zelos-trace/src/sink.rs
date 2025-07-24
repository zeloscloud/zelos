use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;
use zelos_trace_types::ipc::{IpcMessageWithId, Receiver, Sender};

use crate::{filter::Filter, router::DEFAULT_CHANNEL_SIZE};

#[async_trait]
pub(crate) trait TraceSinkHandle: Send + Sync {
    async fn send_async(&self, msg: &IpcMessageWithId) -> Result<()>;
}

/// The handle for a trace sink that has filters
pub(crate) struct TraceSinkHandleFiltered {
    pub sender: Sender,
    pub filters: Arc<RwLock<Vec<Filter>>>,
}

#[async_trait]
impl TraceSinkHandle for TraceSinkHandleFiltered {
    async fn send_async(&self, msg: &IpcMessageWithId) -> Result<()> {
        for filter in self.filters.read().await.iter() {
            if filter.matches(msg) {
                self.sender.try_send(msg.clone())?;
                continue;
            }
        }
        Ok(())
    }
}

pub(crate) struct TraceSinkHandleAllBlocking {
    pub sender: Sender,
}

impl TraceSinkHandleAllBlocking {
    pub(crate) fn new() -> (Self, Receiver) {
        let (sender, receiver) = flume::bounded::<IpcMessageWithId>(DEFAULT_CHANNEL_SIZE);
        (Self { sender }, receiver)
    }
}

#[async_trait]
impl TraceSinkHandle for TraceSinkHandleAllBlocking {
    async fn send_async(&self, msg: &IpcMessageWithId) -> Result<()> {
        self.sender.send_async(msg.clone()).await?;
        Ok(())
    }
}

/// A trace sink is a client connection for the trace router. It hold state about what data the client has seen and is
/// subscribed to.
#[derive(Debug)]
pub struct TraceSink {
    /// The list of filters for this sink
    filters: Arc<RwLock<Vec<Filter>>>,
}

impl TraceSink {
    /// Create a new TraceSink and TraceSinkHandle pair that share a set of filters
    pub(crate) fn new() -> (Self, Receiver, TraceSinkHandleFiltered) {
        let (sender, receiver) = flume::bounded::<IpcMessageWithId>(1024);
        let filters = Arc::new(RwLock::new(Vec::new()));
        (
            Self {
                filters: filters.clone(),
            },
            receiver,
            TraceSinkHandleFiltered { sender, filters },
        )
    }

    /// Add `filter` to the list of filters for this sink
    pub async fn subscribe(&self, filter: Filter) {
        let mut filters = self.filters.write().await;
        filters.push(filter);
    }

    /// Remove `filter` from the list of filters for this sink
    pub async fn unsubscribe(&self, filter: Filter) {
        let mut filters = self.filters.write().await;
        filters.retain(|f| f != &filter);
    }
}
