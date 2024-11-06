use lambda_apigw_utils::SimpleError;
use thiserror::Error;

use crate::Tattoo;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sheep is not in the shed: {0}")]
    SheepNotPresent(Tattoo),
    #[error("Sheep already in the shed: {0}")]
    SheepDuplicationError(Tattoo),
    #[error("Generic error: {0}")]
    GenericError(String),
}

impl From<Error> for SimpleError {
    fn from(value: Error) -> Self {
        lambda_apigw_utils::lambda_commons_utils::log::error!("sheep_shed::Error: {value}");
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
