use std::future::Future;
use std::pin::Pin;

use axum::handler::Handler;
use axum::http::header::{CACHE_CONTROL, HeaderValue};
use axum::http::request::Request;
use axum::response::Response;

pub mod sql {
    use diesel::expression::functions::define_sql_function;
    use diesel::sql_types::Integer;

    define_sql_function!(fn abs(x: Integer) -> Integer);
}

pub const fn cache_forever<H>(handler: H) -> CacheControlMiddleware<H> {
    CacheControlMiddleware::new(handler)
}

#[derive(Debug, Clone)]
pub struct CacheControlMiddleware<H> {
    next_handler: H,
}

impl<H> CacheControlMiddleware<H> {
    const CACHE_CONTROL_VALUE: &str = "public, max-age=31536000, immutable";

    pub const fn new(next_handler: H) -> Self {
        Self { next_handler }
    }
}

impl<T, S, B, H> Handler<T, S, B> for CacheControlMiddleware<H>
where
    H: Handler<T, S, B>,
    S: Send + 'static,
    B: Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, req: Request<B>, state: S) -> Self::Future {
        Box::pin(async move {
            let mut response = self.next_handler.call(req, state).await;
            response.headers_mut().insert(
                CACHE_CONTROL,
                HeaderValue::from_static(Self::CACHE_CONTROL_VALUE),
            );
            response
        })
    }
}
