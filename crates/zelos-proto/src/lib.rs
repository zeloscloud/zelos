pub mod actions;
pub mod channel;
pub mod error;
pub mod trace;

/// Maximum gRPC message size (100MB)
/// This limit is applied to both encoding and decoding across all gRPC clients and services
pub const MAX_GRPC_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
