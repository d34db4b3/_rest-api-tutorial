use actix_web::{
    body::BoxBody,
    http::{self},
    HttpResponse, ResponseError,
};
use std::fmt;

#[derive(Debug)]
pub enum ContentTypeError {
    UnsupportedMediaType,
    Other(actix_web::error::Error),
}

impl fmt::Display for ContentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ResponseError for ContentTypeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ContentTypeError::UnsupportedMediaType => http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ContentTypeError::Other(e) => e.as_response_error().status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ContentTypeError::UnsupportedMediaType => HttpResponse::new(self.status_code()),
            ContentTypeError::Other(e) => e.error_response(),
        }
    }
}
