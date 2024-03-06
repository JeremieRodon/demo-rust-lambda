use aws_sdk_dynamodb::Client;
use dynamodb_sheep_shed::DynamoDBSheepShed;
use rand::Rng;
use sheep_shed::{Sheep, SheepShed, Tatoo, Weight};

use lambda_apigw_commons::prelude::*;

/// Create a random weight for a Sheep between 80 and 160 kg
async fn generate_random_weight() -> Weight {
    let min = Weight::MIN.as_ug();
    let max = Weight::MAX.as_ug();
    Weight::from_ug(rand::thread_rng().gen_range(min..max))
}

async fn insert_sheep(req: SimpleRequest<'_>) -> SimpleResult {
    let parameters = req.parameters;
    let tatoo_parameter = *parameters
        .get("Tatoo")
        .expect("API Gateway ensures it's here");
    let tatoo = Tatoo(tatoo_parameter.parse().map_err(|e| {
        SimpleError::InvalidInput(format!(
            "Tatoo parameter {tatoo_parameter} could not be parsed: {e}"
        ))
    })?);

    log::info!("tatoo={tatoo:?}");

    let handle = tokio::runtime::Handle::current();

    log::info!("spawning sheep generation...");
    // Random number generation in a separate task
    let new_sheep = handle.spawn(async {
        Sheep {
            tatoo,
            weight: generate_random_weight().await,
        }
    });

    log::info!("create a shed instance");
    let mut dynamodb_sheep_shed = DynamoDBSheepShed::new(dynamo());

    log::info!("waiting sheep generation...");
    let new_sheep = new_sheep.await.unwrap();
    let response = json!(new_sheep);

    log::info!("inserting sheep");
    handle
        .spawn_blocking(move || dynamodb_sheep_shed.add_sheep(new_sheep))
        .await
        .unwrap()?;

    log::info!("success");
    simple_response!(201, response)
}

lambda_main!(insert_sheep, dynamo(DYNAMO) = Client);
