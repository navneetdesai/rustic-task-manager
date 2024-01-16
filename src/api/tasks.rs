
use actix_web::{get, post, put, error::ResponseError, web::Json, web::Path, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use derive_more::{Display};
use log::log;
use crate::model::task::{Task, TaskStatus};
use crate::database::db::DB;

#[derive(Deserialize, Serialize, Debug)]
pub struct TaskIdentifier {
    pub task_id: String,
}

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    title: String,
    description: String,
}

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Task not found")]
    NotFound,
    #[display(fmt = "Task already exists")]
    AlreadyExists,
    #[display(fmt = "Database error")]
    DatabaseError,
    BadRequest,
}


impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::AlreadyExists => StatusCode::BAD_REQUEST,
            Error::DatabaseError => StatusCode::FAILED_DEPENDENCY,
            Error::BadRequest => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

#[get("/tasks/{task_id}")]
pub async fn get_tasks(task_id: Path<TaskIdentifier>, db: Data<DB>) -> Result<Json<Task>, Error> {
    let task = db.get_task(task_id.into_inner().task_id).await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(Error::NotFound)
    }
}

#[post("/task")]
pub async fn submit_task(
    ddb_repo: Data<DB>,
    request: Json<SubmitTaskRequest>
) -> Result<Json<TaskIdentifier>, Error> {
    log::info!("submit_task: {:?}", request.title);
    let task = Task::new (
        request.user_id.clone(),
        request.title.clone(),
        request.description.clone(),
    );

    let task_identifier = task.get_id();
    println!("task_id: {}", task_identifier);
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier { task_id: task_identifier })),
        Err(_) => Err(Error::DatabaseError)
    }
}

#[put("/task/{task_id}/start")]
pub async fn start_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::InProgress,
    ).await
}


#[put("/task/{task_global_id}/cancel")]
pub async fn fail_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::Cancelled,
    ).await
}

#[put("/task/{task_global_id}/complete")]
pub async fn complete_task(
    ddb_repo: Data<DB>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, Error> {
    change_state(
        ddb_repo,
        task_identifier.into_inner().task_id,
        TaskStatus::Done,
    ).await
}

async fn change_state(
    ddb_repo: Data<DB>,
    task_global_id: String,
    new_status: TaskStatus,
) -> Result<Json<TaskIdentifier>, Error> {
    let mut task = match ddb_repo.get_task(
        task_global_id
    ).await {
        Some(task) => task,
        None => return Err(Error::NotFound)
    };

    if !task.can_transition_to(&new_status) {
        return Err(Error::BadRequest);
    };

    task.status = new_status;

    let task_identifier = task.get_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier { task_id: task_identifier })),
        Err(_) => Err(Error::DatabaseError)
    }
}