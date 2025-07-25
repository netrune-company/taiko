use crate::request::FromRequest;
use crate::response::IntoResponse;
use crate::{Handler, Request, Response};
use http::{Method, StatusCode};
use http_body_util::Full;
use hyper::body::Bytes;
use matchit::Match;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq, Hash, Default)]
struct RouteId(u32);

type MethodHandler<S> = dyn Fn(Request, S) -> Pin<Box<dyn Future<Output=Response> + Send + 'static>>
+ Send
+ Sync
+ 'static;

struct Endpoint<S> {
    methods: HashMap<Method, Arc<MethodHandler<S>>>,
}

#[derive(Default)]
pub struct Router<S>
{
    inner: matchit::Router<RouteId>,
    routes: HashMap<RouteId, Endpoint<S>>,
    next_id: RouteId,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: matchit::Router::new(),
            routes: HashMap::new(),
            next_id: RouteId(0),
        }
    }

    fn get_next_id(&mut self) -> RouteId {
        let id = self.next_id.clone();
        self.next_id = RouteId(id.0 + 1);
        id
    }

    pub fn post<I, O>(
        self,
        path: &str,
        handler: impl Handler<I, S, Output=O> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        I: FromRequest<S> + Send + 'static,
        O: IntoResponse + 'static,
    {
        self.insert(path, Method::POST, handler)
    }

    pub fn put<I, O>(
        self,
        path: &str,
        handler: impl Handler<I, S, Output=O> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        I: FromRequest<S> + Send + 'static,
        O: IntoResponse + 'static,
    {
        self.insert(path, Method::PUT, handler)
    }

    pub fn get<I, O>(
        self,
        path: &str,
        handler: impl Handler<I, S, Output=O> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        I: FromRequest<S> + Send + 'static,
        O: IntoResponse + 'static,
    {
        self.insert(path, Method::GET, handler)
    }

    pub fn insert<I, O>(
        mut self,
        path: &str,
        method: Method,
        handler: impl Handler<I, S, Output=O> + Clone + Send + Sync + 'static,
    ) -> Self
    where
        I: FromRequest<S> + Send + 'static,
        O: IntoResponse + 'static,
    {
        let id = match self.inner.at(path) {
            Ok(Match {
                   value: existing_id, ..
               }) => existing_id.clone(),
            Err(_) => {
                let new_id = self.get_next_id();
                self.inner.insert(path, new_id.clone()).unwrap();
                new_id
            }
        };

        let method_handler: Arc<MethodHandler<S>> = Arc::new(move |request, state| {
            let handler = Arc::new(handler.clone());
            Box::pin(async move {
                match I::from_request(request, &state).await {
                    Ok(input) => handler
                        .handle(input, state.clone())
                        .await
                        .into_response(),
                    Err(error) => error.into_response()
                }
            })
        });

        let endpoint = self.routes.entry(id.clone()).or_insert_with(|| Endpoint {
            methods: HashMap::new(),
        });

        if endpoint
            .methods
            .insert(method.clone(), method_handler)
            .is_some()
        {
            panic!("Route `{path}` already has handler for method `{method}`");
        }

        self
    }
}

impl<S> Handler<Request, S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Output = Response;
    type Future = Pin<Box<dyn Future<Output=Self::Output> + Send>>;

    fn handle(&self, mut req: Request, state: S) -> Self::Future {
        let (method, path) = (req.method().clone(), req.uri().path().to_string());
        let state = state.clone();

        let result = {
            let mut response = Response::new(Full::new(Bytes::new()));
            *response.status_mut() = StatusCode::NOT_FOUND;

            if let Ok(Match {
                          value: route_id,
                          params,
                      }) = self.inner.at(&path)
            {
                if let Some(endpoint) = self.routes.get(route_id) {
                    if let Some(handler) = endpoint.methods.get(&method) {
                        let params = params
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect::<HashMap<String, String>>();

                        Ok((handler.clone(), params))
                    } else {
                        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                        Err(response)
                    }
                } else {
                    Err(response)
                }
            } else {
                Err(response)
            }
        };

        Box::pin(async move {
            match result {
                Ok((h, params)) => {
                    req.extensions_mut().insert(params);
                    h(req, state).await
                }
                Err(res) => res,
            }
        })
    }
}