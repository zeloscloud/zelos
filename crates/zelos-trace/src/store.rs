use anyhow::Result;
use zelos_trace_types::ipc;

use crate::TraceMetadata;

pub trait Store: Send + Sync {
    /// Returns the metadata for this store as a vec of ipc messages
    fn metadata_as_ipc(&self) -> Result<Vec<ipc::IpcMessageWithId>>;

    /// Updates this store with an ipc message
    fn update(&self, msg: &ipc::IpcMessageWithId) -> Result<()>;
}

pub struct MetadataOnlyStore {
    metadata: TraceMetadata,
}

impl MetadataOnlyStore {
    pub fn new() -> Self {
        Self {
            metadata: TraceMetadata::new(),
        }
    }
}

impl Store for MetadataOnlyStore {
    fn metadata_as_ipc(&self) -> Result<Vec<ipc::IpcMessageWithId>> {
        Ok(self.metadata.as_ipc())
    }

    fn update(&self, msg: &ipc::IpcMessageWithId) -> Result<()> {
        self.metadata.update(msg);
        Ok(())
    }
}

impl Default for MetadataOnlyStore {
    fn default() -> Self {
        Self::new()
    }
}
