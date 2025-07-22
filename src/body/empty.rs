use crate::request::FromRequest;
use crate::response::IntoResponse;
use crate::{Request, Response};
use http_body_util::Full;
use hyper::body::Bytes;

pub struct Empty;

impl IntoResponse for Empty {
    fn into_response(self) -> Response {
        Response::new(Full::new(Bytes::new()))
    }
}

impl<S> FromRequest<S> for Empty
where
    S: Clone + Send + Sync + 'static,
{
    type Error = Empty;

    async fn from_request(_: Request, _: &S) -> Result<Self, Self::Error> {
        Ok(Empty)
    }
}