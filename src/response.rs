use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use http_body_util::Full;
use hyper::body::Bytes;
use serde::Serialize;

pub type Response = http::Response<Full<Bytes>>;

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        *response.status_mut() = self.0;
        response
    }
}

pub struct Json<T>(pub T)
where
    T: Serialize;

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut response = Response::new(
            Full::new(
                Bytes::from(
                    serde_json::to_vec(&self.0).unwrap()
                )
            )
        );

        response
            .headers_mut()
            .append(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        response
    }
}