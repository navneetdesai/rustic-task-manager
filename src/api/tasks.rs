
use actix_web::{get, error::ResponseError, web::Json, web::Path, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use derive_more::{Display};
use crate::model::task::Task;
use crate::model::task::TaskStatus;
use crate::database::db::DB;

#[derive(Deserialize, Serialize)]
pub struct TaskIdentifier {
    task_id: String,
}

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Task not found")]
    NotFound,
    #[display(fmt = "Task already exists")]
    AlreadyExists,
    #[display(fmt = "Database error")]
    DatabaseError,
}


impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::AlreadyExists => StatusCode::BAD_REQUEST,
            Error::DatabaseError => StatusCode::FAILED_DEPENDENCY
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

#[get("/tasks/{id}")]
pub async fn get_tasks(task_id: Path<TaskIdentifier>, db: Data<DB>) -> Result<Json<Task>, Error> {
    let task = db.get_task(task_id.into_inner().task_id).await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(Error::NotFound)
    }
}

