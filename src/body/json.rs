use crate::request::FromRequest;
use crate::response::IntoResponse;
use crate::{Request, Response};
use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Json<T>(pub T);

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

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Clone + Send + Sync + 'static,
{
    type Error = JsonError;

    async fn from_request(request: Request, _: &S) -> Result<Self, Self::Error> {
        let bytes = request
            .collect()
            .await
            .map_err(|_| JsonError)?
            .to_bytes();

        Ok(Json(serde_json::from_slice(bytes.as_ref()).map_err(|_| JsonError)?))
    }
}

pub struct JsonError;
impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        let mut response = Response::new(Full::new(Bytes::new()));
        *response.status_mut() = StatusCode::BAD_REQUEST;
        response
    }
}
