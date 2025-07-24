use std::{pin::Pin, sync::Arc, time::Duration};

use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use zelos_proto::trace::{
    SubscribeRequest, SubscribeResponse,
    subscribe_request::Cmd,
    trace_subscribe_server::{TraceSubscribe, TraceSubscribeServer},
};
use zelos_trace::{TraceRouter, filter::Filter};

const CHUNK_SIZE: usize = 1024;
const CHUNK_TIMEOUT: Duration = Duration::from_millis(10);

pub struct TraceSubscribeService {
    router: Arc<TraceRouter>,
}

impl TraceSubscribeService {
    pub fn new(router: Arc<TraceRouter>) -> Self {
        Self { router }
    }

    pub fn server(self) -> TraceSubscribeServer<Self> {
        TraceSubscribeServer::new(self)
    }
}

#[tonic::async_trait]
impl TraceSubscribe for TraceSubscribeService {
    type SubscribeStream =
        Pin<Box<dyn Stream<Item = Result<SubscribeResponse, Status>> + Send + 'static>>;

    // TODO(jbott): handle disconnects
    async fn subscribe(
        &self,
        request: Request<Streaming<SubscribeRequest>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        // Attach to our router, forwarding trace messages to the client using a stream
        let (sink, stream) = self
            .router
            .subscribe_stream()
            .await
            .map_err(|e| Status::internal(format!("Failed to subscribe: {}", e)))?;

        let stream = stream.chunks_timeout(CHUNK_SIZE, CHUNK_TIMEOUT).map(|m| {
            Ok(SubscribeResponse::from_ipc(
                m.into_iter().map(|msg| msg.into()).collect(),
            ))
        });

        // Handle messages from the client
        let mut req_stream = request.into_inner();
        tokio::task::spawn(async move {
            while let Some(req) = req_stream.message().await? {
                if let Some(cmd) = req.cmd {
                    match cmd {
                        Cmd::Subscribe(subscribe) => {
                            let filter = match &subscribe.filter {
                                Some(f) => Filter::parse(f),
                                None => Ok(Filter::any()),
                            };

                            match filter {
                                Ok(f) => sink.subscribe(f).await,
                                Err(e) => tracing::error!("Failed to parse filter: {}", e),
                            }
                        }
                        Cmd::Unsubscribe(unsubscribe) => {
                            let filter = match &unsubscribe.filter {
                                Some(f) => Filter::parse(f),
                                None => Ok(Filter::any()),
                            };

                            match filter {
                                Ok(f) => sink.unsubscribe(f).await,
                                Err(e) => tracing::error!("Failed to parse filter: {}", e),
                            }
                        }
                    }
                }
            }

            Ok::<_, anyhow::Error>(())
        });

        Ok(Response::new(Box::pin(stream)))
    }
}
