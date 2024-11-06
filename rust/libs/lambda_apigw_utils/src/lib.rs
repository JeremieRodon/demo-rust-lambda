pub use lambda_commons_utils::aws_lambda_events::apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyResponse,
};
pub use lambda_http;
use lambda_http::{
    http::{
        header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
        HeaderMap,
    },
    Body, Error,
};
use serde_json::{json, Value};

use std::collections::HashMap;

pub mod errors;
pub use errors::SimpleError;

// Re-export other crate that will always be usefull in Lambda functions
pub use lambda_commons_utils;
pub use lambda_commons_utils::aws_config;
pub use lambda_commons_utils::serde_json;

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
pub fn extract_cognito_values(event: &ApiGatewayProxyRequest) -> Option<CognitoValues> {
    if let Some(Value::Object(claims)) = event.request_context.authorizer.fields.get("claims") {
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
    None
}

/// Given an immutable reference to a [Request], returns a [HashMap] structure
/// of [HashMap<&str, &str>]
/// Returns [None] if Cognito claims are not present.
pub fn extract_parameters(event: &ApiGatewayProxyRequest) -> HashMap<&str, &str> {
    let mut parameters = HashMap::default();
    parameters.extend(event.query_string_parameters.iter());
    parameters.extend(
        event
            .path_parameters
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str())),
    );
    parameters
}

pub fn extract_body(event: &ApiGatewayProxyRequest) -> &str {
    event.body.as_ref().map(|s| s.as_str()).unwrap_or("")
}

pub fn standard_response(
    simple_response: SimpleResponse,
) -> Result<ApiGatewayProxyResponse, Error> {
    let SimpleResponse { code, body } = simple_response;
    let status_code = code as i64;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        ACCESS_CONTROL_ALLOW_ORIGIN,
        std::env::var("ALLOW_ORIGIN")
            .expect("Mandatory environment variable `ALLOW_ORIGIN` is not set")
            .parse()
            .unwrap(),
    );

    let body = match body {
        Some(body) => Some(Body::Text(body.to_string())),
        None => Some(Body::Empty),
    };
    Ok(ApiGatewayProxyResponse {
        status_code,
        headers: headers.clone(),
        multi_value_headers: headers,
        body,
        is_base64_encoded: false,
    })
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
            SimpleError::ServerError(_) | SimpleError::InvalidState(_) => 500,
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

#[macro_export(local_inner_macros)]
macro_rules! sync_or_async {
    (sync $blk:block) => {
        tokio::task::spawn_blocking(move || $blk).await.unwrap()
    };
    (async $blk:block) => {
        $blk.await
    };
}

#[macro_export(local_inner_macros)]
macro_rules! lambda_main_internal {
    ($sync_or_async:ident, $rc:ident, $with_auth:literal $(,$fn_name:ident = $sdk:ty)*) => {
        async fn http_function_handler(
            event: $crate::ApiGatewayProxyRequest,
        ) -> Result<
            $crate::ApiGatewayProxyResponse,
            $crate::lambda_http::Error,
        > {
            $crate::lambda_commons_utils::log::info!("{event:?}");

            let lambda_result = $crate::sync_or_async!($sync_or_async {
                // Extract Cognito values from the claims of the token
                let cognito_values = $crate::extract_cognito_values(&event);
                $crate::lambda_commons_utils::log::debug!("cognito_values={cognito_values:?}");
                if $with_auth {
                    // User_id is in fact "sub" so it cannot be absent unless there is no auth at all
                    if cognito_values.is_none() {
                        $crate::lambda_commons_utils::log::error!("No token could be found");
                        return Err($crate::SimpleError::Unauthorized.into());
                    }
                }

                let parameters = $crate::extract_parameters(&event);
                $crate::lambda_commons_utils::log::debug!("parameters={parameters:?}");

                let body = $crate::extract_body(&event);
                $crate::lambda_commons_utils::log::debug!("body={body}");

                let simple_request = $crate::SimpleRequest {
                    cognito_values,
                    parameters,
                    body,
                };
                $crate::lambda_commons_utils::log::debug!("simple_request={simple_request:?}");
                $rc(simple_request)
            });

            $crate::lambda_commons_utils::log::debug!("lambda_result={lambda_result:?}");
            match lambda_result {
                Ok(simple_response) => $crate::standard_response(simple_response),
                Err(simple_error) => $crate::standard_response(simple_error.into()),
            }
        }
        $crate::lambda_commons_utils::lambda_main!(
            async http_function_handler($crate::ApiGatewayProxyRequest)->$crate::ApiGatewayProxyResponse
            $(, $fn_name = $sdk)*
        );
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
/// Following arguments are in the form `<fct_name> = <SDKClient>` and asks for AWS SDK initialization.
/// - `fct_name` is the function name with which you will retrieve the Client
/// - `SDKClient` is the type of the AWS SDK Client you wish to be returned with the function
///
/// All in all, this macro allows each lambda code to focus on the actual job it needs to do rather
/// than on writing the same boiler-plate over and over in a project.
/// # Example
///
/// ```
/// use lambda_apigw_utils::prelude::*;
/// use serde_json::json;
/// fn echo_process(req: SimpleRequest<'_>) -> SimpleResult {
///    let parameters = req.parameters;
///    let body = req.body;
///
///    let client = dynamo();
///
///    simple_response!(200, json!({"parameters": parameters, "body": body}))
/// }
///
/// lambda_main!(echo_process, dynamo = aws_sdk_dynamodb::Client);
/// ```
macro_rules! lambda_main {
    ($sync_or_async:ident $rc:ident $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main_internal!($sync_or_async, $rc, false $(, $fn_name = $sdk)*);
    };
    ($rc:ident $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main_internal!(sync, $rc, false $(, $fn_name = $sdk)*);
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
/// Following arguments are in the form `<fct_name> = <SDKClient>` and asks for AWS SDK initialization.
/// - `fct_name` is the function name with which you will retrieve the Client
/// - `SDKClient` is the type of the AWS SDK Client you wish to be returned with the function
///
/// All in all, this macro allows each lambda code to focus on the actual job it needs to do rather
/// than on writing the same boiler-plate over and over in a project.
/// # Example
///
/// ```
/// use lambda_apigw_utils::prelude::*;
/// use serde_json::json;
/// fn echo_process(req: SimpleRequest<'_>) -> SimpleResult {
///    let parameters = req.parameters;
///    let body = req.body;
///
///    let client = dynamo();
///
///    simple_response!(200, json!({"parameters": parameters, "body": body}))
/// }
///
/// auth_lambda_main!(echo_process, dynamo = aws_sdk_dynamodb::Client);
/// ```
macro_rules! auth_lambda_main {
    ($sync_or_async:ident $rc:ident $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main_internal!($sync_or_async, $rc, true $(, $fn_name = $sdk)*);
    };
    ($rc:ident $(,$fn_name:ident = $sdk:ty)*) => {
        $crate::lambda_main_internal!(sync, $rc, true $(, $fn_name = $sdk)*);
    };
}

pub mod prelude {
    pub use super::lambda_commons_utils::log;
    pub use super::serde_json::{self, json};
    pub use super::{
        auth_lambda_main, lambda_main, simple_response, SimpleError, SimpleRequest, SimpleResult,
    };
}
