use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimpleError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid body schema")]
    InvalidBody,
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

impl From<sheep_shed::errors::Error> for SimpleError {
    fn from(value: sheep_shed::errors::Error) -> Self {
        use sheep_shed::errors::Error;
        log::error!("sheep_shed::Error: {value}");
        match value {
            Error::SheepDuplicationError(_) => SimpleError::InvalidInput(value.to_string()),
            Error::SheepNotPresent(_) => SimpleError::Custom {
                code: 404,
                message: value.to_string(),
            },
            Error::GenericError(_) => Self::ServerError("Please try again later"),
        }
    }
}
