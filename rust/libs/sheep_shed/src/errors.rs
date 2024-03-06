use thiserror::Error;

use crate::Tatoo;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sheep is not in the shed: {0}")]
    SheepNotPresent(Tatoo),
    #[error("Sheep already in the shed: {0}")]
    SheepDuplicationError(Tatoo),
    #[error("Generic error: {0}")]
    GenericError(String),
}
