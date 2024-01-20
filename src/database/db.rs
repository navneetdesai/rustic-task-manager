use crate::model::task::{Task, TaskStatus};
use aws_config::Config;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;
use log::error;
use std::collections::HashMap;
use std::str::FromStr;

/// Database client
/// This is a wrapper around the AWS DynamoDB client
/// It is responsible for all interactions with the database
///
pub struct DB {
    client: Client,
    table_name: String,
}

/// Database error
#[derive(Debug)]
pub struct DatabaseError;

///
/// Fetch the value of a key from a HashMap
/// If the key is not present, return DatabaseError.
///
fn get_value(
    key: &str,
    item: &HashMap<String, AttributeValue>,
) -> Result<String, DatabaseError> {
    match extract_value(key, item) {
        Ok(Some(value)) => Ok(value),
        Ok(None) | Err(DatabaseError) => Err(DatabaseError),
    }
}

///
/// Fetch the value of a key from a HashMap
/// If the key is not present, return None.
fn extract_value(
    key: &str,
    item: &HashMap<String, AttributeValue>,
) -> Result<Option<String>, DatabaseError> {
    match item.get(key) {
        Some(value) => match value.as_s() {
            Ok(val) => Ok(Some(val.clone())),
            Err(_) => Err(DatabaseError),
        },
        None => Ok(None),
    }
}


///
/// Create a task from entries in the hashmap
fn create_task_from_entry(item: &HashMap<String, AttributeValue>) -> Result<Task, DatabaseError> {
    let status: TaskStatus =
        match TaskStatus::from_str(get_value("status", item)?.as_str()) {
            Ok(value) => value,
            Err(_) => return Err(DatabaseError),
        };

    let description = extract_value("description", item)
        .expect("Failed to get description");

    Ok(Task {
        user_id: get_value("sK", item)?,
        task_id: get_value("pK", item)?,
        title: get_value("title", item)?,
        description: description.unwrap_or(String::from("")),
        status,
    })
}

/// Database implementation
impl DB {
    /// Create a new database client for
    /// a table.
    pub fn new(table_name: String, config: Config) -> DB {
        let client = Client::new(&config);
        DB { client, table_name }
    }

    /// Puts a task in dynamodb
    pub async fn put_task(&self, task: Task) -> Result<(), DatabaseError> {
        let request = self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("pK", AttributeValue::S(String::from(task.task_id)))
            .item("sK", AttributeValue::S(String::from(task.user_id)))
            .item("title", AttributeValue::S(String::from(task.title)))
            .item(
                "description",
                AttributeValue::S(String::from(task.description)),
            )
            .item("status", AttributeValue::S(task.status.to_string()));

        match request.send().await {
            Ok(_) => Ok(()),
            Err(x) => {
                println!("{:?}", x);
                Err(DatabaseError)
            }
        }
    }


    /// Fetches a task from dynamodb
    pub async fn get_task(&self, id: String) -> Option<Task> {
        let tokens: Vec<String> = id.split("_").map(|x| String::from(x)).collect();
        let task_uuid = AttributeValue::S(tokens[1].clone());

        let res = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pK = :task_uuid")
            .expression_attribute_names("#pK", "pK")
            .expression_attribute_values(":task_uuid", task_uuid)
            .send()
            .await;
        return match res {
            Ok(output) => match output.items {
                Some(items) => {
                    let item = &items.first()?;
                    error!("{:?}", &item);
                    match create_task_from_entry(item) {
                        Ok(task) => Some(task),
                        Err(_) => None,
                    }
                }
                None => None,
            },
            Err(error) => {
                error!("{:?}", error);
                None
            }
        };
    }
}
