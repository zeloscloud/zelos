//! Shared gRPC channel/endpoint configuration.
//!
//! Centralizes keepalive, timeout, and HTTP/2 window settings so every
//! crate that opens a tonic Channel uses the same parameters.

use std::time::Duration;

use tonic::transport::{Channel, Endpoint};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);
const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RPC_TIMEOUT: Duration = Duration::from_secs(300);

/// Build an `Endpoint` with the project-wide defaults: HTTP/2 keepalive,
/// `MAX_GRPC_MESSAGE_SIZE` window sizes, 5s connect / 5m RPC timeouts.
pub fn default_endpoint_config(
    url: impl Into<String>,
) -> Result<Endpoint, tonic::transport::Error> {
    Ok(Endpoint::from_shared(url.into())?
        .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
        .keep_alive_timeout(KEEP_ALIVE_TIMEOUT)
        .keep_alive_while_idle(true)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(RPC_TIMEOUT)
        .tcp_nodelay(true)
        .http2_adaptive_window(true)
        .initial_connection_window_size(Some(crate::MAX_GRPC_MESSAGE_SIZE as u32))
        .initial_stream_window_size(Some(crate::MAX_GRPC_MESSAGE_SIZE as u32)))
}

/// Build a lazy-connected channel from the given URL using the defaults.
pub fn create_channel(url: impl Into<String>) -> Result<Channel, tonic::transport::Error> {
    Ok(default_endpoint_config(url)?.connect_lazy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_endpoint_config_accepts_valid_urls() {
        assert!(default_endpoint_config("grpc://localhost:2300").is_ok());
        assert!(default_endpoint_config("http://127.0.0.1:2300").is_ok());
    }

    #[test]
    fn default_endpoint_config_rejects_invalid_urls() {
        assert!(default_endpoint_config("").is_err());
        assert!(default_endpoint_config("not a valid url").is_err());
    }
}
