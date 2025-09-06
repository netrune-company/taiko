use std::fmt::{Display, Formatter};
use crate::request::{Consume, Request};
use crate::response::{IntoResponse};
use crate::Response;
use http_body_util::Full;
use hyper::body::Bytes;

pub struct Empty;

impl IntoResponse for Empty {
    fn into_response(self) -> Response {
        Response::new(Full::new(Bytes::new()))
    }
}

impl Display for Empty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Consume for Empty {
    type Error = Empty;

    #[allow(clippy::manual_async_fn)]
    fn consume(_: Request) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static {
        async move { Ok(Empty) }
    }
}
