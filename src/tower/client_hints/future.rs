use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll, ready},
};

use http::{Request, Response};
use pin_project_lite::pin_project;
use tower::{BoxError, Service, util::Oneshot};
use wreq::Body;

use super::service::Exchange;

pin_project! {
    /// Response future for [`ClientHints`].
    ///
    /// [`ClientHints`]: super::ClientHints
    pub struct ResponseFuture<S>
    where
        S: Service<Request<Body>>,
    {
        #[pin]
        state: State<S>,
        exchange: Option<Exchange<S>>,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<S>
    where
        S: Service<Request<Body>>,
    {
        Called {
            #[pin]
            future: S::Future,
        },
        Restarted {
            #[pin]
            future: Oneshot<S, Request<Body>>,
        },
    }
}

impl<S> ResponseFuture<S>
where
    S: Service<Request<Body>>,
{
    // Create a new [`ResponseFuture`]
    #[inline]
    pub(super) fn new(future: S::Future, exchange: Option<Exchange<S>>) -> Self {
        ResponseFuture {
            state: State::Called { future },
            exchange,
        }
    }
}

impl<S, B> Future for ResponseFuture<S>
where
    S: Service<Request<Body>, Response = Response<B>>,
    S::Error: Into<BoxError>,
{
    type Output = Result<Response<B>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let res = match this.state.as_mut().project() {
                StateProj::Called { future } => ready!(future.poll(cx)),
                StateProj::Restarted { future } => ready!(future.poll(cx)),
            }
            .map_err(Into::into)?;

            let Some((service, req)) = this
                .exchange
                .as_mut()
                .and_then(|exchange| exchange.on_response(&res))
            else {
                return Poll::Ready(Ok(res));
            };

            // Dropping the response abandons its body, as Chrome does.
            drop(res);
            this.state.set(State::Restarted {
                future: Oneshot::new(service, req),
            });
        }
    }
}

impl<S> fmt::Debug for ResponseFuture<S>
where
    S: Service<Request<Body>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResponseFuture").finish()
    }
}
