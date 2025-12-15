use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use tokio::time::Sleep;
use tower::BoxError;

pin_project! {
    /// Response future for [`Delay`].
    ///
    /// [`Delay`]: super::Delay
    #[derive(Debug)]
    pub struct ResponseFuture<S> {
        #[pin]
        response: S,
        #[pin]
        sleep: Sleep,
    }
}

impl<S> ResponseFuture<S> {
    // Create a new [`ResponseFuture`]
    pub(crate) fn new(response: S, sleep: Sleep) -> Self {
        ResponseFuture { response, sleep }
    }
}

impl<F, S, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<S, E>>,
    E: Into<BoxError>,
{
    type Output = Result<S, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // First poll the sleep until complete
        match this.sleep.poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(_) => {}
        }

        // Then poll the inner future
        match this.response.poll(cx) {
            Poll::Ready(v) => Poll::Ready(v.map_err(Into::into)),
            Poll::Pending => Poll::Pending,
        }
    }
}
