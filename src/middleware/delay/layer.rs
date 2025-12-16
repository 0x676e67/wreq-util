use std::{
    task::{Context, Poll},
    time::Duration,
};

use tower::{BoxError, Layer, Service};

use super::future::ResponseFuture;

/// A Tower Layer that introduces a fixed delay before each request.
#[derive(Clone, Debug)]
pub struct DelayLayer {
    delay: Duration,
}

impl DelayLayer {
    /// Create a new [`DelayLayer`] with the given delay duration.
    #[inline]
    pub const fn new(delay: Duration) -> Self {
        DelayLayer { delay }
    }
}

impl<S> Layer<S> for DelayLayer {
    type Service = Delay<S>;

    #[inline]
    fn layer(&self, service: S) -> Self::Service {
        Delay::new(service, self.delay)
    }
}

/// A Tower [`Service`] that introduces a fixed delay before each request.
#[derive(Debug, Clone)]
pub struct Delay<S> {
    inner: S,
    delay: Duration,
}

impl<S> Delay<S> {
    /// Create a new [`Delay`] service wrapping the given inner service
    #[inline]
    pub fn new(inner: S, delay: Duration) -> Self {
        Delay { inner, delay }
    }
}

impl<S, Request> Service<Request> for Delay<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = ResponseFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let response = self.inner.call(req);
        let sleep = tokio::time::sleep(self.delay);
        ResponseFuture::new(response, sleep)
    }
}
