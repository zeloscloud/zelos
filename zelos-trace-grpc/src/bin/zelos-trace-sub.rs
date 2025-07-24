use anyhow::Result;
use clap::Parser;
use tokio_util::sync::CancellationToken;
use zelos_trace_grpc::subscribe::TraceSubscribeClient;
use zelos_trace_types::ipc::IpcMessageWithId;

#[derive(Debug, Parser)]
struct Args {
    /// The host to connect to
    #[arg(short = 'H', long, default_value = "localhost:2300")]
    host: String,

    /// The filter to apply to the subscription
    #[arg(short, long)]
    filter: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create utilities for receiving messages from the server
    let (sender, receiver) = flume::bounded::<IpcMessageWithId>(100);
    let shutdown = CancellationToken::new();

    // Hook up a signal handler to ctrl-c
    {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await?;
            shutdown.cancel();
            Ok::<(), anyhow::Error>(())
        })
    };

    // Connect to the server
    let addr = format!("grpc://{}", args.host);
    eprintln!("[*] Connecting to gRPC server at {}", addr);
    let (client, task_client) = TraceSubscribeClient::new(sender, shutdown.clone(), addr).await?;
    let client_task = tokio::spawn(task_client);

    // Send a subscription request for the specified filter (or None)
    if let Some(ref f) = args.filter {
        eprintln!("[*] Using filter: {}", f);
    }
    client.subscribe(args.filter, None).await?;

    // Print out messages as we receive them
    while let Ok(ipc) = receiver.recv() {
        println!("{:?}", ipc);
    }
    eprintln!("[*] Response stream ended");

    // Shut down the client task
    shutdown.cancel();
    client_task.await??;

    Ok(())
}
