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
