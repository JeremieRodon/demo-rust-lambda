pub use lambda_http;
use lambda_http::{request::RequestContext, Body, Error, Request, RequestExt, Response};
use serde_json::{json, Value};

use std::collections::HashMap;

pub mod errors;
pub use errors::SimpleError;

// Re-export other crate that will always be usefull in Lambda functions
pub use aws_config;
pub use env_logger;
pub use log;
pub use serde_json;
pub use tokio;

/// Structure containing references on important values from the Amazon Cognito
/// token as extracted by API Gateway using Proxy integration
#[derive(Debug)]
pub struct CognitoValues<'a> {
    /// The Cognito UserId, `sub` field of the OIDC token
    pub user_id: &'a str,
    /// The Cognito email, if the `email` field is present
    pub email: Option<&'a str>,
    /// The Cognito username, if the `cognito:username` field is present
    pub username: Option<&'a str>,
}

/// Given an immutable reference to a [Request], returns a [CognitoValues] structure
/// if the [Request] was received from an API Gatway with Lambda proxy integration.
/// Returns [None] if Cognito claims are not present.
pub fn extract_cognito_values(event: &Request) -> Option<CognitoValues> {
    if let Some(RequestContext::ApiGatewayV1(api_gateway)) = event.request_context_ref() {
        if let Some(Value::Object(claims)) = api_gateway.authorizer.fields.get("claims") {
            return Some(CognitoValues {
                user_id: claims
                    .get("sub")
                    .expect("sub is always present")
                    .as_str()
                    .expect("sub is always a String"),
                email: claims
                    .get("email")
                    .filter(|&v| v.is_string())
                    .map(|v| v.as_str().expect("just tested it's a string")),
                username: claims
                    .get("cognito:username")
                    .filter(|&v| v.is_string())
                    .map(|v| v.as_str().expect("just tested it's a string")),
            });
        }
    }
    None
}

/// Given an immutable reference to a [Request], returns a [HashMap] structure
/// of [HashMap<&str, &str>]
/// Returns [None] if Cognito claims are not present.
pub fn extract_parameters(event: &Request) -> HashMap<&str, &str> {
    let mut parameters = HashMap::default();
    if let Some(query_map) = event.query_string_parameters_ref() {
        parameters.extend(query_map.iter());
    }
    if let Some(query_map) = event.path_parameters_ref() {
        parameters.extend(query_map.iter());
    }
    parameters
}

pub fn extract_body(event: &Request) -> &str {
    std::str::from_utf8(event.body()).expect("invalid utf-8 sequence")
}

pub fn standard_response(simple_response: SimpleResponse) -> Result<Response<Body>, Error> {
    let SimpleResponse { code, body } = simple_response;
    let builder = Response::builder()
        .status(code)
        .header("content-type", "application/json")
        .header(
            "Access-Control-Allow-Origin",
            std::env::var("ALLOW_ORIGIN")
                .expect("Mandatory environment variable `ALLOW_ORIGIN` is not set"),
        );
    let response = match body {
        Some(body) => builder.body(Body::Text(body.to_string())),
        None => builder.body(Body::Empty),
    };
    Ok(response.map_err(Box::new)?)
}

#[derive(Debug)]
pub struct SimpleRequest<'a> {
    // The Cognito values, if the resquest is authenticated by Cognito/APIGateway
    pub cognito_values: Option<CognitoValues<'a>>,
    // Query parameters and path parameters
    pub parameters: HashMap<&'a str, &'a str>,
    // Raw body
    pub body: &'a str,
}

#[derive(Debug)]
pub struct SimpleResponse {
    pub code: u16,
    pub body: Option<Value>,
}
#[macro_export]
macro_rules! simple_response {
    ($code:literal) => {
        Ok($crate::SimpleResponse {
            code: $code,
            body: None,
        })
    };
    ($code:literal, $($body:tt)+) => {
        Ok($crate::SimpleResponse {
            code: $code,
            body: Some($($body)+),
        })
    };
}

impl From<SimpleError> for SimpleResponse {
    fn from(value: SimpleError) -> Self {
        let code = match value {
            SimpleError::InvalidInput(_) | SimpleError::InvalidBody => 400,
            SimpleError::NotFound { .. } => 404,
            SimpleError::Unauthorized => 401,
            SimpleError::ServerError(_) => 500,
            SimpleError::Custom { code, message } => {
                return SimpleResponse {
                    code,
                    body: Some(json!({"message": message})),
                }
            }
        };
        SimpleResponse {
            code,
            body: Some(json!({"message": value.to_string()})),
        }
    }
}

pub type SimpleResult = Result<SimpleResponse, SimpleError>;

#[macro_export]
macro_rules! lambda_main_internal {
    ($rc:ident, $with_auth:literal $(,$fn_name:ident($name:ident) = $sdk:ty)*) => {
        async fn function_handler(
            event: $crate::lambda_http::Request,
        ) -> Result<
            $crate::lambda_http::Response<$crate::lambda_http::Body>,
            $crate::lambda_http::Error,
        > {
            $crate::log::info!("{event:?}");
            let cognito_values = if $with_auth {
                // Extract Cognito values from the claims of the token
                let cognito_values = $crate::extract_cognito_values(&event);
                $crate::log::debug!("cognito_values={cognito_values:?}");
                // User_id is in fact "sub" so it cannot be absent unless there is no auth at all
                if cognito_values.is_none() {
                    $crate::log::error!("No token could be found");
                    return Err($crate::SimpleError::Unauthorized.into());
                }
                cognito_values
            } else {
                None
            };
            let parameters = $crate::extract_parameters(&event);
            $crate::log::debug!("parameters={parameters:?}");
            let body = $crate::extract_body(&event);
            $crate::log::debug!("body={body}");
            let simple_request = $crate::SimpleRequest {
                cognito_values,
                parameters,
                body,
            };
            $crate::log::debug!("simple_request={simple_request:?}");

            match $rc(simple_request).await {
                Ok(simple_response) => $crate::standard_response(simple_response),
                Err(simple_error) => $crate::standard_response(simple_error.into()),
            }
        }

        $(
            static $name : std::sync::OnceLock<$sdk> = std::sync::OnceLock::new();
            fn $fn_name() -> $sdk {
                $name.get().unwrap().clone()
            }
        )*

        use $crate::tokio;
        use $crate::log;
        #[tokio::main]
        async fn main() -> Result<(), $crate::lambda_http::Error> {
            $crate::env_logger::Builder::from_env(
                $crate::env_logger::Env::default()
                    .default_filter_or("info,tracing::span=warn")
                    .default_write_style_or("never"),
            )
            .format_timestamp_micros()
            .init();

            let mut sdk_config: Option<$crate::aws_config::SdkConfig> = None;
            $(
                if sdk_config.is_none() {
                    sdk_config = Some($crate::aws_config::load_from_env().await);
                }
                $name.set(<$sdk>::new(sdk_config.as_ref().unwrap())).expect("cannot fail");
            )*
            $crate::lambda_http::run($crate::lambda_http::service_fn(function_handler)).await
        }
    };
}

#[macro_export]
/// This macro writes the boiler-plate code for lambda that **DON'T NEED** to ensure that the
/// original API Gateway call was authenticated with Cognito.
///
/// If you need the [CognitoValues] extracted and included in the [SimpleRequest], see [auth_lambda_main].
///
/// It writes the boiler-plate code common to most of (if not all) the Lambda functions of the project
/// The first argument is the functional entry-point that will receive a preprocessed [SimpleRequest] and must
/// return a [SimpleResult].
///
/// Following arguments are in the form `<fct_name>(<static>) = <SDKClient>` and asks for AWS SDK initialization.
/// - `fct_name` is the function name with which you will retrieve the Client
/// - `static` is the name of the thread-safe static holding the Client (you will never use it, but you must provide it for technical reasons)
/// - `SDKClient` is the type of the AWS SDK Client you wish to be returned with the function
///
/// All in all, this macro allows each lambda code to focus on the actual job it needs to do rather
/// than on writing the same boiler-plate over and over in a project.
/// # Example
///
/// ```
/// use lambda_apigw_commons::prelude::*;
/// async fn echo_process(req: SimpleRequest<'_>) -> SimpleResult {
///    let parameters = req.parameters;
///    let body = req.body;
///
///    simple_response!(200, json!({"parameters": parameters, "body": body}))
/// }
///
/// use aws_sdk_dynamodb::Client;
/// lambda_main!(echo_process, dynamo(DYNAMO) = Client);
/// ```
macro_rules! lambda_main {
    ($rc:ident $(,$fn_name:ident($name:ident) = $sdk:ty)*) => {
        $crate::lambda_main_internal!($rc, false $(, $fn_name($name) = $sdk)*);
    };
}

#[macro_export]
/// This macro writes the boiler-plate code for lambda that **NEEDS** to ensure that the
/// original API Gateway call was authenticated with Cognito.
///
/// If you don't need the [CognitoValues] extracted and included in the [SimpleRequest], see [lambda_main].
///
/// It writes the boiler-plate code common to most of (if not all) the Lambda functions of the project
/// The first argument is the functional entry-point that will receive a preprocessed [SimpleRequest] and must
/// return a [SimpleResult].
///
/// Following arguments are in the form `<fct_name>(<static>) = <SDKClient>` and asks for AWS SDK initialization.
/// - `fct_name` is the function name with which you will retrieve the Client
/// - `static` is the name of the thread-safe static holding the Client (you will never use it, but you must provide it for technical reasons)
/// - `SDKClient` is the type of the AWS SDK Client you wish to be returned with the function
///
/// All in all, this macro allows each lambda code to focus on the actual job it needs to do rather
/// than on writing the same boiler-plate over and over in a project.
/// # Example
///
/// ```
/// use lambda_apigw_commons::prelude::*;
/// async fn echo_process(req: SimpleRequest<'_>) -> SimpleResult {
///    let parameters = req.parameters;
///    let body = req.body;
///
///    simple_response!(200, json!({"parameters": parameters, "body": body}))
/// }
///
/// use aws_sdk_dynamodb::Client;
/// lambda_main!(echo_process, dynamo(DYNAMO) = Client);
/// ```
macro_rules! auth_lambda_main {
    ($rc:ident) => {
        $crate::lambda_main_internal!($rc, true);
    };
}

pub mod prelude {
    pub use super::log;
    pub use super::serde_json::json;
    pub use super::{lambda_main, simple_response, SimpleError, SimpleRequest, SimpleResult};
}
