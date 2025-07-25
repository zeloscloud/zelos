#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Missing the data type field")]
    MissingDataType,

    #[error("Missing the value field")]
    MissingValue,

    #[error("Missing a message value")]
    MissingMessage,

    #[error("Missing a oneof value")]
    MissingOneOf,

    #[error("Invalid UUID")]
    InvalidUuid(#[from] uuid::Error),

    #[error("Truncation error")]
    IntTruncationError(#[from] std::num::TryFromIntError),
}
