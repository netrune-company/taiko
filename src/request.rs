use crate::response::IntoResponse;
use crate::Response;
use hyper::body::Incoming;
use crate::extract::Extract;

pub type Request = http::Request<Incoming>;

pub trait Consume<S>: Sized {
    type Error: IntoResponse + Send + Sync + 'static;

    fn consume(request: Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static;
}

impl<S> Consume<S> for Request
where
    S: Clone + Send + Sync + 'static,
{
    type Error = Response;

    #[allow(clippy::manual_async_fn)]
    fn consume(request: Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static {
        async move { Ok(request) }
    }
}

pub trait RequestExt<S> {
    fn extract<E>(&self, state: &S) -> impl Future<Output=Result<E, E::Error>>
    where
        E: Extract<S>;


    fn body<B>(self, state: &S) -> impl Future<Output=Result<B, B::Error>>
    where
        B: Consume<S>;
}

impl<S> RequestExt<S> for Request {
    fn extract<E>(&self, state: &S) -> impl Future<Output=Result<E, E::Error>>
    where
        E: Extract<S>
    {
        E::extract(self, state)
    }

    fn body<B>(self, state: &S) -> impl Future<Output=Result<B, B::Error>>
    where
        B: Consume<S>
    {
        B::consume(self, state)
    }
}
