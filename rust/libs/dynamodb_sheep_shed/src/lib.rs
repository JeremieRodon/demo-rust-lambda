use std::cmp::min;

use aws_sdk_dynamodb::{
    error::ProvideErrorMetadata,
    operation::{delete_item::DeleteItemError, put_item::PutItemError},
    types::{ReturnValue, Select},
    Client,
};
use serde_dynamo::{aws_sdk_dynamodb_1::from_item, to_attribute_value, to_item};
use sheep_shed::{Sheep, SheepShed, Tattoo};

/// A [SheepShed] that rely on a DynamoDB database
/// # Important note
/// It is expected that it is always use in the context of
/// a [tokio::runtime::Runtime] of the multi_thread kind as this will heavily
/// rely on calling [tokio::runtime::Handle::current].
///
/// Also, the calls to the method of the [SheepShed] trait MUST always
/// be called with [tokio::task::spawn_blocking].
///
/// Note that it is of course MUCH easier to just stay in the async context,
/// and for this pet project we could easily keep it all "async" if the
/// SheepShed trait was async. That being said, in real life Traits from
/// external crates are often defined as "sync" so making this
/// async -> sync -> async bridge is unfortunately difficult to avoid at
/// some point (and async Trait are only like a month old anyway).
#[derive(Debug)]
pub struct DynamoDBSheepShed {
    client: Client,
    table_name: String,
}

impl DynamoDBSheepShed {
    /// Creates a new [DynamoDBSheepShed] from a [Client] and a `table_name`
    /// # Panics
    /// Panics if called outside of a [tokio] context.
    pub fn new(client: Client) -> Self {
        let table_name = std::env::var("BACKEND_TABLE_NAME")
            .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
        log::info!("BACKEND_TABLE_NAME={table_name}");
        DynamoDBSheepShed::local_new(client, table_name)
    }

    fn local_new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    async fn _full_table_scan(
        &self,
        count_only: bool,
    ) -> Result<(usize, Option<Vec<Sheep>>), sheep_shed::errors::Error> {
        log::info!("_full_table_scan(count_only={count_only})");
        // Request the approximate item count that DynamoDB updates sometimes
        let approx_table_size = self
            .client
            .describe_table()
            .table_name(self.table_name.as_str())
            .send()
            .await
            .map_err(|e| {
                let dte = e.into_service_error();
                let err_string = format!("{dte} ({:?}: {:?})", dte.code(), dte.message());
                log::error!("{err_string}");
                sheep_shed::errors::Error::GenericError(err_string)
            })?
            .table
            .expect("table must exist")
            .item_count
            .unwrap_or_default();

        log::info!("approx_table_size={approx_table_size}");

        // The database representation of a Sheep is ~50 bytes
        // Therefore, a scan page of 1MB will contain ~20 000 sheeps
        // We will be very conservative and devide the table into 100k sheeps segments because
        // it would be very counter-productive to have more parallel scans than we would have had pages.
        // Bottom line, with 100k items per segment we expect each segment to be fully scanned in 5 requests.
        let parallel_scan_threads = min(1 + approx_table_size / 100_000, 1_000_000) as i32;

        let handle = tokio::runtime::Handle::current();
        let futures = (0..parallel_scan_threads)
            .into_iter()
            .map(|seg| {
                let client = self.client.clone();
                let table_name = self.table_name.clone();
                handle.spawn(async move {
                    let mut items = if !count_only { Some(vec![]) } else { None };
                    let mut count = 0;
                    let mut exclusive_start_key = None;
                    loop {
                        let result = client
                            .scan()
                            .table_name(&table_name)
                            .segment(seg)
                            .total_segments(parallel_scan_threads)
                            .set_exclusive_start_key(exclusive_start_key)
                            .select(if count_only {
                                Select::Count
                            } else {
                                Select::AllAttributes
                            })
                            .send()
                            .await
                            .map_err(|e| {
                                let se = e.into_service_error();
                                let err_string =
                                    format!("{se} ({:?}: {:?})", se.code(), se.message());
                                log::error!("{err_string}");
                                sheep_shed::errors::Error::GenericError(err_string)
                            })?;
                        exclusive_start_key = result.last_evaluated_key;
                        count += result.count as usize;
                        if !count_only {
                            items.as_mut().unwrap().extend(
                                result.items.unwrap_or_default().into_iter().map(|i| {
                                    from_item(i).expect("cannot fail unless database corrupt")
                                }),
                            );
                        }
                        if exclusive_start_key.is_none() {
                            break;
                        }
                    }
                    Ok((count, items))
                })
            })
            .collect::<Vec<_>>();

        log::info!("Launched {} green-threads", futures.len());

        let mut items = if !count_only { Some(vec![]) } else { None };
        let mut count = 0;
        for f in futures {
            let (sub_count, sub_vec) = f.await.unwrap()?;
            count += sub_count;
            if !count_only {
                items.as_mut().unwrap().extend(sub_vec.unwrap().into_iter());
            }
        }
        log::info!(
            "_full_table_scan => Ok(({count}, {}))",
            if items.is_some() { "Some" } else { "None" }
        );
        Ok((count, items))
    }

    async fn _sheep_iter_impl(
        &self,
    ) -> Result<impl Iterator<Item = Sheep>, sheep_shed::errors::Error> {
        Ok(self._full_table_scan(false).await?.1.unwrap().into_iter())
    }
    async fn _sheep_count_impl(&self) -> Result<usize, sheep_shed::errors::Error> {
        Ok(self._full_table_scan(true).await?.0)
    }
    async fn _add_sheep_impl(&self, sheep: Sheep) -> Result<(), sheep_shed::errors::Error> {
        log::info!("_add_sheep_impl(sheep={sheep})");
        let _ = self
            .client
            .put_item()
            .table_name(self.table_name.as_str())
            .set_item(Some(to_item(&sheep).expect("cannot fail")))
            .condition_expression("attribute_not_exists(tattoo)")
            .send()
            .await
            .map_err(|e| {
                let pie = e.into_service_error();
                match pie {
                    PutItemError::ConditionalCheckFailedException(_) => {
                        sheep_shed::errors::Error::SheepDuplicationError(sheep.tattoo)
                    }
                    _ => {
                        let err_string = format!("{pie} ({:?}: {:?})", pie.code(), pie.message());
                        log::error!("{err_string}");
                        sheep_shed::errors::Error::GenericError(err_string)
                    }
                }
            })?;
        log::info!("_add_sheep_impl => Ok(())");
        Ok(())
    }
    async fn _kill_sheep_impl(&self, tattoo: &Tattoo) -> Result<Sheep, sheep_shed::errors::Error> {
        log::info!("_kill_sheep_impl(tattoo={tattoo})");
        let sheep = self
            .client
            .delete_item()
            .table_name(self.table_name.as_str())
            .key("tattoo", to_attribute_value(tattoo).expect("cannot fail"))
            .condition_expression("attribute_exists(tattoo)")
            .return_values(ReturnValue::AllOld)
            .send()
            .await
            .map_err(|e| {
                let die = e.into_service_error();
                match die {
                    DeleteItemError::ConditionalCheckFailedException(_) => {
                        sheep_shed::errors::Error::SheepNotPresent(tattoo.clone())
                    }
                    _ => {
                        let err_string = format!("{die} ({:?}: {:?})", die.code(), die.message());
                        log::error!("{err_string}");
                        sheep_shed::errors::Error::GenericError(err_string)
                    }
                }
            })?
            .attributes
            .map(|i| from_item(i).expect("cannot fail unless database corrupt"))
            .expect("DynamoDB verified Sheep was present");
        log::info!("_kill_sheep_impl => Ok({sheep})");
        Ok(sheep)
    }
}

impl SheepShed for DynamoDBSheepShed {
    /// Add a new [Sheep] in the [SheepShed]
    /// # Errors
    /// It is not allowed to add a duplicated [Sheep], will return an
    /// [errors::Error::SheepDuplicationError] if the user tries to add
    /// a [Sheep] with an already known [Tattoo]
    /// # Panics
    /// Panics if called outside of a blocking thread ([tokio::task::spawn_blocking]).
    fn add_sheep(&mut self, sheep: Sheep) -> Result<(), sheep_shed::errors::Error> {
        tokio::runtime::Handle::current().block_on(self._add_sheep_impl(sheep))
    }

    /// Return the number of [Sheep] in the [SheepShed]
    /// # Panics
    /// Panics if called outside of a blocking thread ([tokio::task::spawn_blocking]).
    fn sheep_count(&self) -> Result<usize, sheep_shed::errors::Error> {
        tokio::runtime::Handle::current().block_on(self._sheep_count_impl())
    }
    /// Return an [Iterator] over references of all the [Sheep]s in the [SheepShed]
    /// # Panics
    /// Panics if called outside of a blocking thread ([tokio::task::spawn_blocking]).
    fn sheep_iter(&self) -> Result<impl Iterator<Item = Sheep>, sheep_shed::errors::Error> {
        tokio::runtime::Handle::current().block_on(self._sheep_iter_impl())
    }

    fn kill_sheep(
        &mut self,
        tattoo: &sheep_shed::Tattoo,
    ) -> Result<Sheep, sheep_shed::errors::Error> {
        tokio::runtime::Handle::current().block_on(self._kill_sheep_impl(tattoo))
    }
}

// The test module need to have DynamoDB local running
// You can, for example, use the Java version from AWS:
// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.DownloadingAndRunning.html
// I'm using Correto to launch it, here is the command-line (assuming you are in the directory containing the .jar file and the lib folder):
// java -D"java.library.path=./DynamoDBLocal_lib" -jar DynamoDBLocal.jar -sharedDb -inMemory
#[cfg(test)]
mod tests {

    use aws_sdk_dynamodb::types::{
        AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType,
    };

    use super::*;

    fn dynamodb_local_client() -> Client {
        let config = aws_sdk_dynamodb::Config::builder()
            .endpoint_url("http://localhost:8000")
            .behavior_version_latest()
            .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
                "fakeMyKeyId",
                "fakeSecretAccessKey",
                None,
                None,
                "Static",
            ))
            .region(Some(aws_sdk_dynamodb::config::Region::from_static(
                "eu-west-1",
            )))
            .build();
        Client::from_conf(config)
    }

    struct TempTable {
        client: Client,
        table_name: String,
    }

    impl TempTable {
        fn new(client: Client, table_name: &str) -> Self {
            let pkn = "tattoo";
            let pkad = AttributeDefinition::builder()
                .attribute_name(pkn)
                .attribute_type(ScalarAttributeType::N)
                .build()
                .unwrap();
            let pk = KeySchemaElement::builder()
                .attribute_name(pkn)
                .key_type(KeyType::Hash)
                .build()
                .unwrap();
            tokio::runtime::Handle::current()
                .block_on(
                    client
                        .create_table()
                        .table_name(table_name)
                        .attribute_definitions(pkad)
                        .key_schema(pk)
                        .billing_mode(BillingMode::PayPerRequest)
                        .send(),
                )
                .unwrap();

            Self {
                client,
                table_name: table_name.to_owned(),
            }
        }
    }

    impl Drop for TempTable {
        fn drop(&mut self) {
            tokio::runtime::Handle::current()
                .block_on(
                    self.client
                        .delete_table()
                        .table_name(&self.table_name)
                        .send(),
                )
                .ok()
                .or_else(|| None);
        }
    }

    fn prep_base_sheep_shed() -> (TempTable, DynamoDBSheepShed) {
        let client = dynamodb_local_client();
        let table_name = format!("{}", rand::random::<u64>());
        let temp_table = TempTable::new(client.clone(), &table_name);
        let sheep_shed = DynamoDBSheepShed::local_new(client, table_name);
        (temp_table, sheep_shed)
    }

    // The DynamoDBSheepShed expects to be called from an async context
    // In Lambda that make sense because they are launch as async.
    // So because of the async -> sync -> async expectation of the library,
    // we simulate that in our tests by launching them "as-if" it was a Lambda
    // context.
    // Note that it is of course MUCH easier to just stay in the async context,
    // and for this pet project we could easily keep it all "async" if the
    // SheepShed trait was async. That being said, in real life Traits from
    // external crates are often defined as "sync" so making this
    // async -> sync -> async bridge is unfortunately difficult to avoid at
    // some point (and async Trait are only like a month old anyway).
    macro_rules! impl_test_template {
        ($tn: tt) => {
            #[test]
            fn $tn() {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    rt.spawn_blocking(|| {
                        let (_temp, sheep_shed) = prep_base_sheep_shed();
                        sheep_shed::test_templates::$tn(sheep_shed)
                    })
                    .await
                    .unwrap()
                })
            }
        };
    }

    impl_test_template!(cannot_duplicate_sheep);
    impl_test_template!(sheep_shed_sheep_count);
    impl_test_template!(sheep_shed_iterator);
    impl_test_template!(cannot_kill_inexistent_sheep);
}
