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
    
    #[allow(clippy::manual_async_fn)]
    fn from_request(_: Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static {
        async move { Ok(Empty) }
    }
}