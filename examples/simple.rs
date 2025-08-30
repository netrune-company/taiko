use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use taiko::body::JsonError;
use taiko::prelude::*;
use taiko::request::Extract;
use taiko::response::IntoResponse;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not listen to 8080");

    let router = Router::new()
        .get("/users/{test}", index)
        .get("/users/me", index);

    App::new(())
        .handler(router)
        .layer(LogLayer)
        .listen(listener)
        .await;
}

#[derive(Serialize, Deserialize)]
struct Payload {
    message: String
}

#[derive(Serialize, Deserialize)]
enum HttpError {
    BadRequest
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::BadRequest => Json(Payload {
                message: String::from("Bad request")
            }).into_response()
        }
    }
}

impl From<JsonError> for HttpError {
    fn from(value: JsonError) -> Self {
        match value {
            JsonError::Deserialize => HttpError::BadRequest,
            JsonError::Serialize => HttpError::BadRequest
        }
    }
}

struct Identity;

impl<S> Extract<S> for Identity {
    type Error = HttpError;

    fn extract(request: &Request, state: &S) -> impl Future<Output=Result<Self, Self::Error>> {
        async move {
            Ok(Identity)
        }
    }
}

async fn index(request: Request, _state: ()) -> Result<Json<Payload>, HttpError> {
    Ok(Json(Payload {
        message: String::from("hi")
    }))
}

struct LogLayer;
impl<H> Layer<H> for LogLayer {
    type Handler = LogHandler<H>;

    fn wrap(self, handler: H) -> Self::Handler {
        LogHandler(handler)
    }
}

struct LogHandler<H>(H);
impl<S, H> Handler<Request, S> for LogHandler<H>
where
    H: Handler<Request, S>
{
    type Output = H::Output;
    type Future = H::Future;

    fn handle(&self, input: Request, state: S) -> Self::Future {
        println!("{input:?}");
        self.0.handle(input, state)
    }
}