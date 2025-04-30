use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};

lazy_static::lazy_static! {
    static ref TABLE_NAME: String = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    pub static ref PK: String = std::env::var("pk").expect("PK must be set");
    pub static ref SK: String = std::env::var("sk").expect("SK must be set"); //TODO option?
    pub static ref TEXT: String = std::env::var("text").expect("text must be set"); //TODO dyn in another crate?
}

pub async fn connect() -> Result<Client, Error> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    let client = Client::new(&config);

    Ok(client)
}

pub async fn find_all(
    client: &Client,
    iterator: Option<HashMap<String, AttributeValue>>,
) -> Result<
    (
        Vec<HashMap<String, AttributeValue>>,
        Option<HashMap<String, AttributeValue>>,
    ),
    Error,
> {
    match client
        .scan()
        .table_name(TABLE_NAME.as_str())
        .set_exclusive_start_key(iterator)
        .send()
        .await
    {
        Ok(res) => Ok((res.items().to_vec(), res.last_evaluated_key().cloned())),
        Err(e) => {
            println!("Error finding items: {:#?}", e.to_string());
            return Err(Error::from(e));
        }
    }
}

pub async fn put(client: &Client, pk: &str, sk: &str, text: &str) -> Result<(), Error> {
    let put = HashMap::from([
        (PK.clone(), AttributeValue::S(pk.to_string())),
        (SK.clone(), AttributeValue::S(sk.to_string())),
        (TEXT.clone(), AttributeValue::S(text.to_string())),
    ]);

    return match client
        .put_item()
        .table_name(TABLE_NAME.as_str())
        .set_item(Some(put))
        .send()
        .await
    {
        Ok(_) => {
            println!("Item added successfully.");
            Ok(())
        }
        Err(e) => {
            println!("Error upserting item: {:#?}", e);
            Err(Error::from(e))
        }
    };
}

pub async fn delete(client: &Client, pk: &str, sk: &str) -> Result<(), Error> {
    match client
        .delete_item()
        .table_name(TABLE_NAME.as_str())
        .key(PK.as_str(), AttributeValue::S(pk.to_string()))
        .key(SK.as_str(), AttributeValue::S(sk.to_string()))
        .send()
        .await
    {
        Ok(_) => {
            println!("Item deleted successfully.");
            Ok(())
        }
        Err(e) => {
            println!("Error deleting item: {:#?}", e.to_string());
            return Err(Error::from(e));
        }
    }
}
