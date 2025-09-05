use crate::handler::{Boxed, EchoHandler};
use crate::{Handler, Layer, Request, Response};
use hyper::service::Service;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use hyper_util::server::conn::auto::Builder;
use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::TcpListener;
use tokio::{pin, signal, spawn};
use tracing::{error, info};

pub struct App<S, H> {
    state: Arc<S>,
    handler: H,
}

impl<S> App<S, EchoHandler>
{
    pub fn new(state: S) -> App<S, EchoHandler>
    {
        App {
            state: Arc::new(state),
            handler: EchoHandler,
        }
    }

    pub fn handler<H>(self, handler: H) -> App<S, H>
    where
        H: Handler<Request, S>
    {
        App {
            state: self.state,
            handler
        }
    }
}

impl<S, H> App<S, H> {
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
    H::Future: Send,
{
    pub async fn listen(self, listener: TcpListener) {
        info!("Listening...");
        let app = self.boxed();
        let graceful = GracefulShutdown::new();
        let http = Builder::new(TokioExecutor::new());

        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    break;
                }
                Ok((stream, client)) = listener.accept() => {
                    let app = app.clone();
                    let io = TokioIo::new(stream);
                    let connection = graceful.watch(http
                        .serve_connection_with_upgrades(io, AppService(app, client))
                        .into_owned());

                    spawn(async move {
                        if let Err(err) = connection.await {
                            error!("Connection error: {:?}", err);
                        }
                    });
                }
            }
        }

        info!("Waiting for connections to complete...");
        graceful.shutdown().await;
        info!("All connections closed. Shutting down.");
    }
}

impl<S, H> App<S, Boxed<H>>
where
    S: Clone + Send + Sync + 'static,
    H: Handler<Request, S, Output=Response>,
{
    pub fn handle(&self, request: Request) -> H::Future {
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
    H::Future: Send + 'static,
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
