use crate::response::IntoResponse;
use crate::Response;
use hyper::body::Incoming;

pub type Request = http::Request<Incoming>;

pub trait FromRequest<S>: Sized {
    type Error: IntoResponse + Send + Sync + 'static;

    fn from_request(request: Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static;
}

pub trait FromRequestRef<S>: Sized {
    type Error: IntoResponse + Send + Sync + 'static;

    fn from_request_ref(request: &Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static;
}

impl<S> FromRequest<S> for Request
where
    S: Clone + Send + Sync + 'static,
{
    type Error = Response;

    #[allow(clippy::manual_async_fn)]
    fn from_request(request: Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static {
        async move { Ok(request) }
    }
}
