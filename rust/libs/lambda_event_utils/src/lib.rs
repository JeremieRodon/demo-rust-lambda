mod errors;
mod sendevents;
pub use crate::sendevents::send_custom_event;

pub mod prelude {
    pub use lambda_commons_utils::prelude::*;
    pub use lambda_commons_utils::serde_json;
}
