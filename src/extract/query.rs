use std::fmt::{Display, Formatter};
use std::ops::Deref;
use serde::de::DeserializeOwned;
use crate::body::Empty;
use crate::Request;
use crate::request::Extract;

pub struct Query<T>(pub T);

impl<S, T> Extract<S> for Query<T>
where
    T: DeserializeOwned
{
    type Error = QueryError;

    fn extract(request: &Request, _: &S) -> impl Future<Output=Result<Self, Self::Error>> {
        async {
            let query = request.uri().query().unwrap_or_default();

            Ok(
                Query(
                    serde_urlencoded::from_str(query)
                        .map_err(|e| QueryError(e.to_string()))?
                )
            )
        }
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct QueryError(pub String);

impl Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
