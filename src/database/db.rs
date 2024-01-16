use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_config::Config;
use crate::model::task::{Task, TaskStatus};
use log::error;
use std::str::FromStr;
use std::collections::HashMap;

pub struct DB {
    client: Client,
    table_name: String
}

#[derive(Debug)]
pub struct DatabaseError;


fn required_item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<String, DatabaseError> {
    match item_value(key, item) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(DatabaseError),
        Err(DatabaseError) => Err(DatabaseError)
    }
}

fn item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<Option<String>, DatabaseError> {
    match item.get(key) {
        Some(value) => match value.as_s() {
            Ok(val) => Ok(Some(val.clone())),
            Err(_) => Err(DatabaseError)
        },
        None => Ok(None)
    }
}

fn item_to_task(item: &HashMap<String, AttributeValue>) -> Result<Task, DatabaseError> {
    let status: TaskStatus = match TaskStatus::from_str(required_item_value("status", item)?.as_str()) {
        Ok(value) => value,
        Err(_) => return Err(DatabaseError)
    };

    let description = item_value("description", item).expect("Failed to get description");

    Ok(Task {
        user_id: required_item_value("pK", item)?,
        task_id: required_item_value("sK", item)?,
        title: required_item_value("title", item)?,
        description: description.unwrap_or(String::from("")),
        status,
    })
}

impl DB {
    pub fn new(table_name: String, config: Config) -> DB {
        let client = Client::new(&config);
        DB {
            client,
            table_name,
        }
    }

    pub async fn put_task(&self, task: Task) -> Result<(), DatabaseError> {
        println!("{}", &self.table_name);
        let mut request = self.client.put_item()
            .table_name(&self.table_name)
            .item("pK", AttributeValue::S(String::from(task.task_id)))
            .item("sK", AttributeValue::S(String::from(task.user_id)))
            .item("title", AttributeValue::S(String::from(task.title)))
            .item("description", AttributeValue::S(String::from(task.description)))
            .item("status", AttributeValue::S(task.status.to_string()));


        match request.send().await {
            Ok(_) => Ok(()),
            Err(x) => {
                println!("{:?}", x);
                Err(DatabaseError)
            }
        }
    }



    pub async fn get_task(&self, id: String) -> Option<Task> {
        let tokens:Vec<String> = id
            .split("_")
            .map(|x| String::from(x))
            .collect();
        let user_uuid = AttributeValue::S(tokens[0].clone());
        let task_uuid = AttributeValue::S(tokens[1].clone());
        println!("{},{}", tokens[0].clone(), tokens[1].clone());

        let res = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pK = :task_uuid")
            .expression_attribute_names("#pK", "pK")
            .expression_attribute_values(":task_uuid", task_uuid)
            .send()
            .await;
        println!("Query Request: {:?}", res);
        return match res {
            Ok(output) => {
                match output.items {
                    Some(items) => {
                        let item = &items.first()?;
                        error!("{:?}", &item);
                        match item_to_task(item) {
                            Ok(task) => Some(task),
                            Err(_) => None
                        }
                    },
                    None => {
                        None
                    }
                }
            },
            Err(error) => {
                error!("{:?}", error);
                None
            }
        }
    }
}