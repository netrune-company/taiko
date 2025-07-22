use crate::response::IntoResponse;
use crate::Response;
use hyper::body::Incoming;

pub type Request = http::Request<Incoming>;

pub trait FromRequest<S>: Sized {
    type Error: IntoResponse + Send;

    fn from_request(request: Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send;
}

impl<S> FromRequest<S> for Request
where
    S: Clone + Send + Sync + 'static,
{
    type Error = Response;

    async fn from_request(request: Request, _: &S) -> Result<Self, Self::Error> {
        Ok(request)
    }
}
