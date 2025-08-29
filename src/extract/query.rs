use std::ops::Deref;
use serde::de::DeserializeOwned;
use crate::body::Empty;
use crate::extract::Extract;
use crate::Request;

pub struct Query<T>(pub T);

impl<S, T> Extract<S> for Query<T>
where
    T: DeserializeOwned
{
    type Error = Empty;

    fn extract(request: &Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> {
        async {
            let query = request.uri().query().unwrap_or_default();
            Ok(Query(serde_urlencoded::from_str(query).map_err(|_| Empty)?))
        }
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}