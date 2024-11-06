pub use aws_config;
pub use aws_lambda_events;
pub use env_logger;
pub use lambda_runtime;
pub use log;
pub use serde_json;
pub use tokio;

#[macro_export]
macro_rules! sync_or_async {
    (sync $rc:ident($arg:ident)) => {
        tokio::task::spawn_blocking(move || $rc($arg))
            .await
            .unwrap()
    };
    (async $rc:ident($arg:ident)) => {
        $rc($arg).await
    };
}

#[macro_export]
macro_rules! lambda_main {
    (common $sync_or_async:ident $rc:ident($event_type:ty)->$return_type:ty $(, $code:stmt)?) => {
        use $crate::tokio;

        async fn function_handler(
            event: $crate::lambda_runtime::LambdaEvent<$event_type>,
        ) -> Result<$return_type, $crate::lambda_runtime::Error> {
            $crate::log::debug!("{event:?}");
            let payload = event.payload;
            $crate::log::info!("{}", $crate::serde_json::json!(payload));
            Ok($crate::sync_or_async!($sync_or_async $rc(payload))?)
        }


        #[tokio::main]
        async fn main() -> Result<(), $crate::lambda_runtime::Error> {
            $crate::env_logger::Builder::from_env(
                $crate::env_logger::Env::default()
                    .default_filter_or("info,tracing::span=warn")
                    .default_write_style_or("never"),
            )
            .format_timestamp_micros()
            .init();

            $($code;)?

            $crate::lambda_runtime::run($crate::lambda_runtime::service_fn(function_handler)).await
        }
    };
    ($sync_or_async:ident $rc:ident($event_type:ty)->$return_type:ty $(,$fn_name:ident = $sdk:ty)+) => {
        static AWS_SDK_CONFIG : std::sync::OnceLock<$crate::aws_config::SdkConfig> = std::sync::OnceLock::new();
        pub fn aws_sdk_config() -> &'static $crate::aws_config::SdkConfig {
            AWS_SDK_CONFIG.get().unwrap()
        }

        // AWS SDK clients globals
        $(
            pub fn $fn_name() -> $sdk {
                static CLIENT : std::sync::OnceLock<$sdk> = std::sync::OnceLock::new();
                CLIENT.get_or_init(||<$sdk>::new(aws_sdk_config())).clone()
            }
        )+

        $crate::lambda_main!(common $sync_or_async $rc($event_type)->$return_type,
            // AWS SDK clients globals instantiation
            AWS_SDK_CONFIG.set($crate::aws_config::load_from_env().await).unwrap()
        );

    };
    ($sync_or_async:ident $rc:ident($event_type:ty)->$return_type:ty) => {
        $crate::lambda_main!(common $sync_or_async $rc($event_type)->$return_type);
    };
    ($sync_or_async:ident $rc:ident($event_type:ty) $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main!($sync_or_async $rc($event_type)->() $(, $fn_name = $sdk)*);
    };
    ($rc:ident($event_type:ty)->$return_type:ty $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main!(sync $rc($event_type)->$return_type $(, $fn_name = $sdk)*);
    };
    ($rc:ident($event_type:ty) $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main!($rc($event_type)->() $(, $fn_name = $sdk)*);
    };
}

pub mod prelude {
    pub use super::lambda_main;
    pub use super::log;
}
