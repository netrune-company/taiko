use crate::{Request, Response};
use std::pin::Pin;
use std::sync::Arc;

pub trait Layer<H>
{
    type Handler;

    fn wrap(self, handler: H) -> Self::Handler;
}

pub trait Handler<I, S> {
    type Output;

    fn handle(&self, input: I, state: S) -> impl Future<Output=Self::Output> + Send + 'static;
}

impl<F, Fut, I, O, S> Handler<I, S> for F
where
    F: Fn(I, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output=O> + Send + 'static,
{
    type Output = O;

    fn handle(&self, input: I, state: S) -> impl Future<Output=Self::Output> + Send + 'static {
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

    fn handle(&self, input: I, state: S) -> impl Future<Output=Self::Output> + 'static {
        self.0.handle(input, state)
    }
}

pub trait HttpHandler<S> {
    fn handle(&self, request: Request, state: S) -> Pin<Box<dyn Future<Output=Response> + Send + 'static>>;
}

impl<H, S> HttpHandler<S> for H
where
    H: Handler<Request, S, Output=Response>,
{
    fn handle(&self, request: Request, state: S) -> Pin<Box<dyn Future<Output=Response> + Send + 'static>> {
        Box::pin(self.handle(request, state))
    }
}