use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// TaskStatus is an enum that represents the status of a task.
/// A task can be in one of the following states:
/// Todo: The task has not been started yet.
/// InProgress: The task is currently being worked on.
/// Done: The task has been completed.
/// Cancelled: The task has been cancelled.
#[derive(Serialize, Deserialize, EnumString, Display, Eq, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

/// Task is a struct that represents a task.
/// A task has the following fields:
/// user_id: The id of the user that owns the task.
/// task_id: The id of the task.
/// title: The title of the task.
/// description: The description of the task.
/// status: The status of the task.
/// The task_id is generated using the uuid crate.
/// The user_id and task_id are combined to form the id of the task.
/// The id of the task is used as the key in the database.
///
#[derive(Serialize, Deserialize)]
pub struct Task {
    pub user_id: String,
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
}

///
/// Task impl block
///
impl Task {
    /// new is a function that creates a new task.
    /// It takes in the user_id, title and description of the task.
    /// The default status of the task is Todo.
    /// It returns a new task.
    /// The task_id is generated using the uuid crate.
    ///
    pub fn new(user_id: String, title: String, description: String) -> Task {
        Task {
            user_id,
            task_id: Uuid::new_v4().to_string(),
            title,
            description,
            status: TaskStatus::Todo,
        }
    }

    ///
    /// get_id is a function that returns the id of the task.
    /// The id of the task is a combination of the user_id and task_id.
    ///
    pub fn get_id(&self) -> String {
        format!("{}_{}", self.user_id, self.task_id)
    }

    ///
    /// Checks if the task can transition to the given status
    /// from its current status
    ///
    pub fn can_transition_to(&self, status: &TaskStatus) -> bool {
        match (&self.status, status) {
            (TaskStatus::Todo, TaskStatus::InProgress) => true,
            (TaskStatus::InProgress, TaskStatus::Done) => true,
            (TaskStatus::InProgress, TaskStatus::Cancelled) => true,
            _ => false,
        }
    }
}
