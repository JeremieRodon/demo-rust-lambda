use aws_sdk_eventbridge::{types::builders::PutEventsRequestEntryBuilder, Client};

use lambda_commons_utils::log;
use lambda_commons_utils::serde_json::json;

use serde::Serialize;
use serde_type_name::type_name;

use crate::errors::Error;

/// Send an event in the Custom EventBus of the project and return the EventId
///
/// ## Errors
///
/// The function returns error if it fails to send the event to EventBridge
///
/// ## Panics
///
/// The function panics if the EVENT_BUS_NAME env variable is not set
pub async fn send_custom_event<T>(event_bridge_client: Client, event: T) -> Result<String, Error>
where
    T: std::fmt::Debug + Serialize,
{
    log::debug!("send_custom_event - event={event:?}");
    let event_bus_name = std::env::var("EVENT_BUS_NAME")
        .expect("Mandatory environment variable `EVENT_BUS_NAME` is not set");
    let event = PutEventsRequestEntryBuilder::default()
        .source("lambda-event-utils")
        .event_bus_name(event_bus_name)
        .detail_type(type_name(&event).map_err(|_| Error::TypeInferenceFailed)?)
        .detail(json!(event).to_string())
        .build();
    let put_event = event_bridge_client.put_events().entries(event);
    let result = put_event.send().await?;
    let entry_result = result
        .entries()
        .first()
        .expect("vec should always have one entry");
    if result.failed_entry_count > 0 {
        Err(Error::EventBridgeEntryError {
            code: entry_result
                .error_code()
                .expect("should always an error code")
                .to_owned(),
            message: entry_result
                .error_message()
                .expect("should always an error message")
                .to_owned(),
        })
    } else {
        Ok(entry_result
            .event_id()
            .expect("should always an event_id")
            .to_owned())
    }
}
