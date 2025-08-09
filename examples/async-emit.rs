use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::TraceRouter;
use zelos_trace::TraceSource;
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    // Connect publish client to the running agent
    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    let (client, client_task) = TracePublishClient::new(
        router.clone(),
        TracePublishClientConfig {
            url: url.clone(),
            ..Default::default()
        },
    );
    tokio::spawn(client_task);
    client.wait_until_connected(Duration::from_secs(5)).await?;

    let source = TraceSource::new("async-demo", router.sender());
    let evt = source
        .build_event("progress")
        .add_u64_field("step", None)
        .add_string_field("message", None)
        .build()?;

    // Emit using async helpers
    evt.build()
        .try_insert_u64("step", 1)?
        .try_insert_string("message", "starting".into())?
        .emit_async()
        .await?;

    evt.build()
        .try_insert_u64("step", 2)?
        .try_insert_string("message", "working".into())?
        .emit_at_async(zelos_trace::time::now_time_ns())
        .await?;

    Ok(())
}
