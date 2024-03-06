use aws_sdk_dynamodb::Client;
use dynamodb_sheep_shed::DynamoDBSheepShed;
use sheep_shed::SheepShed;

use lambda_apigw_commons::prelude::*;

async fn bark_answer(_req: SimpleRequest<'_>) -> SimpleResult {
    log::info!("create a shed instance");
    let dynamodb_sheep_shed = DynamoDBSheepShed::new(dynamo());

    log::info!("counting sheeps...");
    let count = tokio::runtime::Handle::current()
        .spawn_blocking(move || dynamodb_sheep_shed.sheep_count())
        .await
        .unwrap()?;

    log::info!("success - count={count}");
    simple_response!(200, json!({"count": count}))
}

lambda_main!(bark_answer, dynamo(DYNAMO) = Client);
