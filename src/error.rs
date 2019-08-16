use rocket::http::Status;
use rocket::{response, Request};
use rocket::response::{Responder, status};

pub type ApiResponse<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    AlreadyExists,
    FailedSaving,
    Io(std::io::Error),
}

/// Impl to implicitly convert io errors to our error
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Io(err)
    }
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            ApiError::NotFound => status::NotFound(Some("Entry not found")).respond_to(req),
            ApiError::AlreadyExists => status::BadRequest(Some("The item with the same title already exists")).respond_to(req),
            // TODO: optionally, just let io error enter the generic error below
            ApiError::Io(io) => status::Custom(Status::InternalServerError, io.to_string()).respond_to(req),
            _ => status::Custom(Status::InternalServerError, "Internal server error").respond_to(req),
        }
    }
}
