use std::fmt;

/// Error types for Action operations
#[derive(Debug)]
pub enum ActionsError {
    InvalidJson(String),
    NotFound(String),
    ExecutionError(String),
    InvalidInput(String),
}

impl fmt::Display for ActionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionsError::NotFound(name) => write!(f, "Action not found: {}", name),
            ActionsError::InvalidJson(err) => write!(f, "Invalid JSON: {}", err),
            ActionsError::ExecutionError(err) => write!(f, "Execution error: {}", err),
            ActionsError::InvalidInput(err) => write!(f, "Invalid input: {}", err),
        }
    }
}

impl std::error::Error for ActionsError {}
