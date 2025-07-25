//! Zelos - A distributed tracing system
//!
//! This crate provides a unified interface to the Zelos tracing system,
//! re-exporting the main functionality from the individual crates.

pub use zelos_proto as proto;
pub use zelos_trace as trace;
pub use zelos_trace_grpc as trace_grpc;

// Re-export commonly used types
pub use zelos_trace::{TraceRouter, TraceSink, TraceSource, Store};
pub use zelos_trace_grpc::{publish, subscribe};
