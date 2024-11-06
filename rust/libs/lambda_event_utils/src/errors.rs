use aws_sdk_eventbridge::{error::SdkError, operation::put_events::PutEventsError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("EventBridgeError: {code}/{message}")]
    EventBridgeEntryError { code: String, message: String },
    #[error("Failed to infere the type of the event from the event type")]
    TypeInferenceFailed,
    #[error("EventBridgeError: {source:#}")]
    EventBridgeError {
        #[from]
        source: SdkError<PutEventsError>,
    },
}
