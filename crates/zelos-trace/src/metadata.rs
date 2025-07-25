use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use rpds::HashTrieMapSync;
use uuid::Uuid;
use zelos_trace_types::ipc;

use crate::segment::TraceSegment;

#[derive(Clone)]
pub struct TraceMetadata {
    segments: Arc<ArcSwap<HashTrieMapSync<Uuid, TraceSegment>>>,
}

impl TraceMetadata {
    pub fn new() -> Self {
        Self {
            segments: Arc::new(ArcSwap::from_pointee(HashTrieMapSync::new_sync())),
        }
    }

    pub fn from(msgs: impl IntoIterator<Item = ipc::IpcMessageWithId>) -> Self {
        let mut segments: HashTrieMapSync<Uuid, TraceSegment> = HashTrieMapSync::new_sync();

        for msg in msgs {
            if let Some(seg) = segments.get_mut(&msg.segment_id) {
                match &msg.msg {
                    ipc::IpcMessage::TraceSegmentStart(_)
                    | ipc::IpcMessage::TraceSegmentEnd(_)
                    | ipc::IpcMessage::TraceEventSchema(_)
                    | ipc::IpcMessage::TraceEventFieldNamedValues(_) => seg.update_mut(&msg.msg),
                    ipc::IpcMessage::TraceEvent(_) => {
                        // Do nothing
                    }
                }
            } else {
                let seg = if let ipc::IpcMessage::TraceSegmentStart(m) = &msg.msg {
                    TraceSegment::from_ipc(msg.segment_id, m)
                } else {
                    TraceSegment::empty(msg.segment_id, msg.source_name)
                };
                segments.insert_mut(msg.segment_id, seg.update(&msg.msg));
            }
        }

        Self {
            segments: Arc::new(ArcSwap::from_pointee(segments)),
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn update(&self, msg: &ipc::IpcMessageWithId) {
        // Early return if we see messages we don't care about
        if let ipc::IpcMessage::TraceEvent(_) = &msg.msg {
            return;
        }

        let segments = self.segments.load();
        if let Some(seg) = segments.get(&msg.segment_id) {
            match &msg.msg {
                ipc::IpcMessage::TraceSegmentStart(_)
                | ipc::IpcMessage::TraceSegmentEnd(_)
                | ipc::IpcMessage::TraceEventSchema(_)
                | ipc::IpcMessage::TraceEventFieldNamedValues(_) => {
                    let new = segments.insert(msg.segment_id, seg.update(&msg.msg));
                    self.segments.store(Arc::new(new));
                }
                ipc::IpcMessage::TraceEvent(_) => {
                    // Do nothing
                }
            }
        } else {
            let seg = if let ipc::IpcMessage::TraceSegmentStart(m) = &msg.msg {
                TraceSegment::from_ipc(msg.segment_id, m)
            } else {
                TraceSegment::empty(msg.segment_id, msg.source_name.clone())
            };
            let new = segments.insert(msg.segment_id, seg.update(&msg.msg));
            self.segments.store(Arc::new(new));
        }
    }

    pub fn as_ipc(&self) -> Vec<ipc::IpcMessageWithId> {
        let segments = self.segments.load();
        segments
            .iter()
            .flat_map(|(id, seg)| {
                seg.as_ipc()
                    .into_iter()
                    .map(move |msg| ipc::IpcMessageWithId {
                        segment_id: *id,
                        source_name: seg.source.clone(),
                        msg,
                    })
            })
            .collect()
    }

    /// Returns a clone of all trace segments
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn segments(&self) -> HashMap<Uuid, TraceSegment> {
        let segments = self.segments.load();
        segments
            .iter()
            .map(|(id, seg)| (*id, seg.clone()))
            .collect()
    }

    /// Returns an iterator of a clone of all trace segments
    pub fn segments_iter(&self) -> impl Iterator<Item = TraceSegment> {
        let segments = self.segments.load();
        segments.values().cloned().collect::<Vec<_>>().into_iter()
    }

    /// Returns a clone of a single trace segment
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn get_segment(&self, id: &Uuid) -> Option<TraceSegment> {
        let segments = self.segments.load();
        segments.get(id).cloned()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn remove_segment(&self, id: &Uuid) {
        let segments = self.segments.load();
        let new = segments.remove(id);
        self.segments.store(Arc::new(new));
    }
}

impl Default for TraceMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;

    use super::*;

    #[test]
    fn test_basic() {
        let metadata = TraceMetadata::new();

        let segment_id = Uuid::try_parse("0196c84d-6eb8-7c46-83b1-e4cac73ba9b6").unwrap();
        let source_name = "src";
        let start = ipc::TraceSegmentStart {
            time_ns: 0,
            source_name: source_name.to_string(),
        };
        metadata.update(&ipc::IpcMessageWithId {
            segment_id,
            source_name: source_name.to_string(),
            msg: start.into(),
        });

        {
            let seg = metadata.get_segment(&segment_id).unwrap();
            assert_eq!(seg.id, segment_id);
            assert_eq!(seg.source, "src");
            assert_eq!(seg.start_time, Some(DateTime::from_timestamp_nanos(0)));
            assert_eq!(seg.end_time, None);
        }

        let end = ipc::TraceSegmentEnd { time_ns: 1 };
        metadata.update(&ipc::IpcMessageWithId {
            segment_id,
            source_name: source_name.to_string(),
            msg: end.into(),
        });

        {
            let seg = metadata.get_segment(&segment_id).unwrap();
            assert_eq!(seg.end_time, Some(DateTime::from_timestamp_nanos(1)));
        }
    }
}
