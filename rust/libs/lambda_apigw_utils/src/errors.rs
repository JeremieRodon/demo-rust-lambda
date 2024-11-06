use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimpleError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid body schema")]
    InvalidBody,
    #[error("Invalid application state: {0}")]
    InvalidState(String),
    #[error("{object_type} not found with ID: {id}")]
    NotFound {
        object_type: &'static str,
        id: String,
    },
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Server error: {0}")]
    ServerError(&'static str),
    #[error("Custom error: {code} {message}")]
    Custom { code: u16, message: String },
}
