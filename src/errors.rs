use actix_web::{error, http::StatusCode, BaseHttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum AppError {
    #[display(fmt = "not found")]
    NotFound,
}

impl error::ResponseError for AppError {
    fn error_response(&self) ->  BaseHttpResponse<actix_web::dev::Body> {
        BaseHttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
