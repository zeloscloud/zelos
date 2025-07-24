#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}
