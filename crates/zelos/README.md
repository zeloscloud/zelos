# zelos

Rust SDK for [Zelos](https://zeloscloud.io). Stream hardware data into the Zelos app over gRPC.

The SDK publishes to a running Zelos app or agent. Get the app at <https://zeloscloud.io/download>.

## Install

```bash
cargo add zelos tokio-util
cargo add tokio --features rt-multi-thread,macros,time
```

Requires Rust 1.85 or later.

## Quick start

```rust
use std::f64::consts::PI;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;
use zelos::publish::{TracePublishClient, TracePublishClientConfig};
use zelos::{TraceRouter, TraceSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Router: carries events from sources to the publish client
    let (router, router_task) = TraceRouter::new(CancellationToken::new());
    tokio::spawn(router_task);

    // Publish client: defaults to grpc://localhost:2300
    let config = TracePublishClientConfig::default();
    let (client, client_task) = TracePublishClient::new(router.clone(), config);
    tokio::spawn(client_task);
    client.wait_until_connected(Duration::from_secs(5)).await?;

    // Source and event schema
    let source = TraceSource::new("power_monitor", router.sender());
    let event = source
        .build_event("measurements")
        .add_f64_field("voltage", Some("V".to_string()))
        .add_f64_field("current", Some("A".to_string()))
        .build()?;

    // Stream at 1 kHz
    let start = Instant::now();
    loop {
        let t = start.elapsed().as_secs_f64();
        event
            .build()
            .try_insert_f64("voltage", 12.0 + 0.5 * (2.0 * PI * 50.0 * t).sin())?
            .try_insert_f64("current", 2.0 + 0.2 * (2.0 * PI * 50.0 * t).cos())?
            .emit()?;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
```

Run it, open the app, go to Signals, and plot `power_monitor/measurements.voltage`.

## Reference

| Item | Purpose |
|------|---------|
| `TraceRouter` | Routes events from sources to sinks |
| `TraceSource` | A named data source. Defines event schemas and emits events |
| `publish::TracePublishClient` | Publishes routed events to an app or agent |
| `publish::TracePublishClientConfig` | URL, batch size, batch timeout, reconnect delay |
| `subscribe::TraceSubscribeClient` | Receives live events from an app or agent |
| `Action`, `ActionsRegistry`, `ActionsClient` | Expose callable operations to the agent |

## Links

- Docs: <https://docs.zeloscloud.io/latest/sdk/quickstart/>
- Examples: <https://github.com/zeloscloud/zelos/tree/main/crates/zelos/examples>

## License

MIT or Apache-2.0, at your option.
