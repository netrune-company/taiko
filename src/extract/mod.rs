mod path;
mod query;

use crate::Request;

pub use path::Path;
pub use query::Query;

pub trait Extract<S>: Sized {
    type Error;

    fn extract(
        request: &Request,
        state: &S,
    ) -> impl Future<Output=Result<Self, Self::Error>>;
}
