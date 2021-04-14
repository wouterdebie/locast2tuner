use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum AppError {
    #[display(fmt = "not found")]
    NotFound,
}

impl error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
