use crate::response::IntoResponse;
use crate::Response;
use hyper::body::Incoming;

pub type Request = http::Request<Incoming>;

pub trait FromRequest<S>: Sized {
    type Error: IntoResponse + Send + Sync + 'static;

    fn from_request(request: Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> + Send + 'static;
}

pub trait Extract<S>: Sized {
    type Error;

    fn extract(
        request: &Request,
        state: &S,
    ) -> impl Future<Output=Result<Self, Self::Error>>;
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

pub trait RequestExt<S> {
    fn extract<E>(&self, state: &S) -> impl Future<Output=Result<E, E::Error>>
    where
        E: Extract<S>;
}


pub trait StatelessRequestExt {
    fn body<T>(self) -> impl Future<Output = Result<T, T::Error>>
    where
        T: FromRequest<()>;
}

impl StatelessRequestExt for Request {
    fn body<T>(self) -> impl Future<Output = Result<T, T::Error>>
    where
        T: FromRequest<()>
    {
        T::from_request(self, &())
    }
}

impl<S> RequestExt<S> for Request {
    fn extract<E>(&self, state: &S) -> impl Future<Output=Result<E, E::Error>>
    where
        E: Extract<S>
    {
        E::extract(self, state)
    }
}
