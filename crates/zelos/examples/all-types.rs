use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
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

    let source = TraceSource::new("all-types", router.sender());
    let evt = source
        .build_event("all_types")
        .add_i8_field("i8", None)
        .add_i16_field("i16", None)
        .add_i32_field("i32", None)
        .add_i64_field("i64", None)
        .add_u8_field("u8", None)
        .add_u16_field("u16", None)
        .add_u32_field("u32", None)
        .add_u64_field("u64", None)
        .add_f32_field("f32", None)
        .add_f64_field("f64", None)
        .add_timestamp_ns_field("ts", Some("ns".into()))
        .add_binary_field("bin", None)
        .add_string_field("str", None)
        .add_bool_field("bool", None)
        .build()?;

    evt.build()
        .try_insert_i8("i8", -8)?
        .try_insert_i16("i16", -16)?
        .try_insert_i32("i32", -32)?
        .try_insert_i64("i64", -64)?
        .try_insert_u8("u8", 8)?
        .try_insert_u16("u16", 16)?
        .try_insert_u32("u32", 32)?
        .try_insert_u64("u64", 64)?
        .try_insert_f32("f32", std::f32::consts::PI)?
        .try_insert_f64("f64", std::f64::consts::E)?
        .try_insert_timestamp_ns("ts", zelos_trace::time::now_time_ns())?
        .try_insert_binary("bin", vec![0x01, 0x02, 0x03])?
        .try_insert_string("str", "hello".to_string())?
        .try_insert_bool("bool", true)?
        .emit()?;

    println!("emitted all_types event");
    Ok(())
}
