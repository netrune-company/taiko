use crate::handler::Boxed;
use crate::{Handler, Layer, Request, Response};
use hyper::service::Service;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App<S, H> {
    state: Arc<S>,
    handler: H,
}

impl<S, H> App<S, H>
{
    pub fn new<I, O>(state: S, handler: H) -> App<S, H>
    where
        H: Handler<I, S, Output=O>,
    {
        App {
            state: Arc::new(state),
            handler,
        }
    }

    pub fn layer<L>(self, layer: L) -> App<S, L::Handler>
    where
        L: Layer<H>,
    {
        App {
            state: self.state,
            handler: layer.wrap(self.handler),
        }
    }

    fn boxed<I, O>(self) -> App<S, Boxed<H>>
    where
        H: Handler<I, S, Output=O>,
    {
        App {
            state: self.state,
            handler: Boxed::new(self.handler),
        }
    }
}

impl<S, H> Clone for App<S, Boxed<H>>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl<S, H> App<S, H>
where
    S: Clone + Send + Sync + 'static,
    H: Handler<Request, S, Output=Response> + Send + Sync + 'static,
{
    pub async fn listen(self, listener: TcpListener) {
        let app = self.boxed();
        loop {
            let (stream, client) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let app = app.clone();

            tokio::spawn(async move {
                auto::Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(io, AppService(app, client))
                    .await
                    .unwrap_or_else(|e| eprintln!("Error serving connection: {e:?}"));
            });
        }
    }
}

impl<S, H> App<S, Boxed<H>>
where
    S: Clone + Send + Sync + 'static,
    H: Handler<Request, S, Output=Response>,
{
    pub fn handle(&self, request: Request) -> impl Future<Output=Response> + Send + 'static {
        self.handler.inner().handle(request, self.state.as_ref().clone())
    }
}

#[derive(Clone)]
pub struct AppService<S, H>(App<S, Boxed<H>>, SocketAddr)
where
    S: Clone + Send + Sync + 'static,
    H: Handler<Request, S, Output=Response>;

impl<S, H> Service<Request> for AppService<S, H>
where
    S: Clone + Send + Sync + 'static,
    H: Handler<Request, S, Output=Response>,
{
    type Response = Response;
    type Error = Infallible;
    type Future =
    Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&self, mut req: Request) -> Self::Future {
        req.extensions_mut().insert(self.1);

        let future = self.0.handle(req);
        Box::pin(async move { Ok(future.await) })
    }
}
