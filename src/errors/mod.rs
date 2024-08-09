use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Cerved error")]
    CervedError,
    #[error("Database error")]
    DatabaseError,
    #[error("Deferred")]
    DeferredError,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("S3 error")]
    S3Error,
}

impl ErrorKind {
    pub(crate) fn error_response(&self) -> HttpResponse {
        match self {
            ErrorKind::CervedError => HttpResponse::BadGateway().finish(),
            ErrorKind::DatabaseError => HttpResponse::InternalServerError().finish(),
            ErrorKind::DeferredError => HttpResponse::NotFound().finish(),
            ErrorKind::S3Error => HttpResponse::BadGateway().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        if let Some(kind) = self.0.downcast_ref::<ErrorKind>() {
            kind.error_response()
        } else {
            ErrorKind::InternalServerError.error_response()
        }
    }
}
