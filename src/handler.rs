use std::pin::Pin;
use std::sync::Arc;
use http_body_util::{BodyExt, Full};
use crate::{Request, Response};
use crate::body::Empty;
use crate::response::IntoResponse;

pub trait Layer<H>
{
    type Handler;

    fn wrap(self, handler: H) -> Self::Handler;
}

pub trait Handler<I, S> {
    type Output: Send + Sync;
    type Future: Future<Output=Self::Output> + Send;

    fn handle(&self, input: I, state: S) -> Self::Future;
}

impl<F, Fut, I, O, S> Handler<I, S> for F
where
    F: Fn(I, S) -> Fut + Send + Sync,
    Fut: Future<Output=O> + Send,
    O: Send + Sync,
{
    type Output = O;
    type Future = Fut;

    fn handle(&self, input: I, state: S) -> Self::Future {
        self(input, state)
    }
}

pub struct Boxed<H>(Arc<H>);

impl<H> Boxed<H> {
    pub fn new(handler: H) -> Self {
        Self(Arc::new(handler))
    }

    pub fn inner(&self) -> &H {
        &self.0
    }
}

impl<H> Clone for Boxed<H> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I, S, H> Handler<I, S> for Boxed<H>
where
    H: Handler<I, S> + Send + Sync + 'static,
{
    type Output = H::Output;
    type Future = H::Future;

    fn handle(&self, input: I, state: S) -> Self::Future {
        self.0.handle(input, state)
    }
}

pub struct EchoHandler;

impl<S> Handler<Request, S> for EchoHandler {
    type Output = Response;
    type Future = Pin<Box<dyn Future<Output = Self::Output> + Send + 'static>>;

    fn handle(&self, input: Request, _state: S) -> Self::Future {
        Box::pin(async move {
            let Ok(body) = input.into_body().collect().await else {
                return Empty.into_response()
            };

            Response::new(Full::new(body.to_bytes()))
        })
    }
}