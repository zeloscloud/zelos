use zelos_trace_types::ipc;

use super::TraceMessageBatch;
use crate::error::Error;

impl super::SubscribeResponse {
    pub fn from_ipc(messages: Vec<super::TraceMessage>) -> Self {
        Self {
            msg: Some(super::subscribe_response::Msg::TraceMessageBatch(
                TraceMessageBatch { messages },
            )),
        }
    }

    pub fn as_ipc(self) -> Result<Vec<ipc::IpcMessageWithId>, Error> {
        match self.msg {
            Some(super::subscribe_response::Msg::TraceMessageBatch(msg)) => {
                msg.messages.into_iter().map(|msg| msg.try_into()).collect()
            }
            None => Err(Error::MissingMessage),
        }
    }
}
