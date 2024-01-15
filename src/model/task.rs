use serde::{Deserialize, Serialize};
use uuid::Uuid;
use strum_macros::{EnumString, Display};
#[derive(Serialize, Deserialize, EnumString, Display, Eq, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub user_id: String,
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
}

impl Task {
    pub fn new(user_id: String, title: String, description: String) -> Task {
        Task {
            user_id,
            task_id: Uuid::new_v4().to_string(),
            title,
            description,
            status: TaskStatus::Todo,
        }
    }

    pub fn get_id(&self) -> String {
        format!("{}_{}", self.user_id, self.task_id)
    }




}