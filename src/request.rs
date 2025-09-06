use std::fmt::Display;
use std::future::Future;
use hyper::body::Incoming;
use crate::body::Empty;
use crate::response::IntoResponse;

pub type Request<B = Incoming> = http::Request<B>;

pub trait Consume: Sized {
    type Error: Display + Send + Sync + 'static;

    fn consume(request: Request) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static;
}

impl Consume for Request {
    type Error = Empty;

    #[allow(clippy::manual_async_fn)]
    fn consume(request: Request) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static {
        async move { Ok(request) }
    }
}

pub trait Extract<S>: Sized {
    type Error: Display + Send + Sync + 'static;

    fn extract(
        request: &Request,
        state: &S,
    ) -> impl Future<Output=Result<Self, Self::Error>>;
}
