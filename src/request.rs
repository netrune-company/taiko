use hyper::body::Incoming;

pub type Request = http::Request<Incoming>;

pub trait FromRequest {
    fn from_request(request: Request) -> Self;
}

impl FromRequest for Request {
    fn from_request(request: Request) -> Self {
        request
    }
}