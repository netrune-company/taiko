use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;
use crate::body::{Empty, Json};

pub type HttpResponse = http::Response<Full<Bytes>>;

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> HttpResponse {
        self
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> HttpResponse {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

pub struct Response<T> {
    body: T,
    status: StatusCode,
}

impl<T> IntoResponse for Response<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> HttpResponse {
        let mut response = self.body.into_response();
        *response.status_mut() = self.status;
        response
    }
}

impl<T> Response<T> {
    pub fn with_status(self, status: StatusCode) -> Response<T> {
        Response {
            body: self.body,
            status
        }
    }
}

impl Response<Empty> {
    pub fn empty() -> Self {
        Self {
            body: Empty,
            status: StatusCode::OK,
        }
    }
}

impl<T> Response<Json<T>>
where
    T: serde::Serialize,
{
    pub fn json(value: T) -> Self {
        Self {
            body: Json(value),
            status: StatusCode::OK,
        }
    }
}
