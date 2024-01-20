use crate::database::db::DB;
use crate::model::task::{Task, TaskStatus};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{error::ResponseError, get, post, put, web::Json, web::Path, HttpResponse};
use derive_more::Display;
use serde::{Deserialize, Serialize};

///
/// Task identifier
///
#[derive(Deserialize, Serialize, Debug)]
pub struct TaskIdentifier {
    pub task_id: String,
}

///
/// This is the request body for the submit_task endpoint
///
#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    title: String,
    description: String,
}

///
/// Error types for the API
///
#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Task not found")]
    NotFound,
    #[display(fmt = "Database error")]
    DatabaseError,
    BadRequest,
}

///
/// Implement the ResponseError trait for the Error type
///
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::DatabaseError => StatusCode::FAILED_DEPENDENCY,
            Error::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

///
/// GET task endpoint
/// Returns a task given a task_id in the request
/// path
#[get("/tasks/{task_id}")]
pub async fn get_tasks(task_id: Path<TaskIdentifier>, db: Data<DB>) -> Result<Json<Task>, Error> {
    match db.get_task(task_id.into_inner().task_id).await {
        Some(task) => Ok(Json(task)),
        None => Err(Error::NotFound),
    }
}

///
/// POST task endpoint
/// Creates a new task given a user_id, title and description
/// in the request body
/// Returns a task identifier
#[post("/task")]
pub async fn submit_task(
    ddb_repo: Data<DB>,
    request: Json<SubmitTaskRequest>,
) -> Result<Json<TaskIdentifier>, Error> {
    let task = Task::new(
        request.user_id.clone(),
        request.title.clone(),
        request.description.clone(),
    );

    let task_identifier = task.get_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier {
            task_id: task_identifier,
        })),
        Err(_) => Err(Error::DatabaseError),
    }
}

///
/// PUT task endpoint
/// Marks a task as InProgress given a task_id in the request
/// Returns a task identifier
#[put("/task/{task_id}/start")]
pub async fn start_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::InProgress,
    )
    .await
}

///
/// PUT task endpoint
/// Marks a task as Cancelled given a task_id in the request
///
#[put("/task/{task_id}/cancel")]
pub async fn fail_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::Cancelled,
    )
    .await
}

///
/// PUT task endpoint
/// Marks a task as Done given a task_id in the request
/// Returns a task identifier
///
#[put("/task/{task_id}/complete")]
pub async fn complete_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::Done,
    )
    .await
}

///
/// Helper function to change the state of a task
/// given a task_id and a new status
///

async fn change_state(
    ddb_repo: Data<DB>,
    task_global_id: String,
    new_status: TaskStatus,
) -> Result<Json<TaskIdentifier>, Error> {
    let mut task = ddb_repo
        .get_task(task_global_id)
        .await
        .ok_or(Error::NotFound)?;

    if !task.can_transition_to(&new_status) {
        return Err(Error::BadRequest);
    };

    task.status = new_status;
    let task_identifier = task.get_id();
    ddb_repo
        .put_task(task)
        .await
        .map_err(|_| Error::DatabaseError)?;
    Ok(Json(TaskIdentifier {
        task_id: task_identifier,
    }))
}
