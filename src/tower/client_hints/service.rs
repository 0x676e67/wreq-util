use std::{
    mem,
    sync::Arc,
    task::{Context, Poll},
};

use http::{Request, Response};
use tower::{BoxError, Service};
use wreq::{
    Body, Method, ResponseExt, Uri,
    header::{HeaderMap, HeaderName, USER_AGENT},
    redirect::History,
};

use super::{
    future::ResponseFuture,
    hints::{Hints, SEC_CH_UA},
    order::HintOrder,
    parse::listed,
    store::{Accepted, Store, origin},
};

const ACCEPT_CH: HeaderName = HeaderName::from_static("accept-ch");
const CRITICAL_CH: HeaderName = HeaderName::from_static("critical-ch");

/// A Tower [`Service`] that sends the client hints an origin accepts, and restarts a request
/// for the critical ones it lacks.
///
/// Created via [`ClientHintsLayer`].
///
/// [`ClientHintsLayer`]: super::ClientHintsLayer
#[derive(Clone, Debug)]
pub struct ClientHints<S> {
    inner: S,
    hints: Option<Arc<Hints>>,
    store: Store,
}

/// State kept to handle the response to one request.
pub(super) struct Exchange<S> {
    hints: Arc<Hints>,
    store: Store,
    /// The origin the request was sent to.
    origin: String,
    /// The hints that origin had asked for when the request was sent.
    accepted: Option<Accepted>,
    /// A copy of the request without client hints, and a service to send it again with.
    restart: Option<(S, Request<Body>)>,
}

// ===== impl ClientHints =====

impl<S> ClientHints<S> {
    /// Creates a new [`ClientHints`] service wrapping the given inner service.
    #[inline]
    pub(super) fn new(inner: S, hints: Option<Arc<Hints>>, store: Store) -> Self {
        ClientHints {
            inner,
            hints,
            store,
        }
    }

    /// Returns the hints of the browser `req` looks like.
    fn hints_for(&self, req: &Request<Body>) -> Option<Arc<Hints>> {
        // A request with its own `Sec-CH-UA` has its own identity, as with a per-request
        // emulation. One that only replaces the `User-Agent` has no matching hints, and Chrome
        // doesn't send any for an overridden user agent either.
        if req.headers().contains_key(SEC_CH_UA) {
            return Hints::from_headers(req.headers()).map(Arc::new);
        }
        if req.headers().contains_key(USER_AGENT) {
            return None;
        }
        self.hints.clone()
    }
}

impl<S, ResBody> Service<Request<Body>> for ClientHints<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = ResponseFuture<S>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let Some((hints, origin)) = self.hints_for(&req).zip(origin(req.uri())) else {
            return ResponseFuture::new(self.inner.call(req), None);
        };
        let accepted = self.store.get(&origin);

        // Keep a copy to restart with. Chrome doesn't send a `POST` again, and a streaming body
        // can't be copied.
        let req = wreq::Request::from(req);
        let copy = matches!(*req.method(), Method::GET | Method::HEAD)
            .then(|| req.try_clone())
            .flatten()
            .map(Request::from);
        let mut req = Request::from(req);
        attach(
            &mut req,
            &hints,
            accepted.as_deref().unwrap_or_default(),
            false,
        );

        // The ready service sends this request and is kept for the restart.
        let (future, restart) = match copy {
            Some(copy) => {
                let clone = self.inner.clone();
                let mut service = mem::replace(&mut self.inner, clone);
                (service.call(req), Some((service, copy)))
            }
            None => (self.inner.call(req), None),
        };

        let exchange = Exchange {
            hints,
            store: self.store.clone(),
            origin,
            accepted,
            restart,
        };
        ResponseFuture::new(future, Some(exchange))
    }
}

// ===== impl Exchange =====

impl<S> Exchange<S> {
    /// Records the hints a response, and the redirects that led to it, ask for.
    ///
    /// Returns the request to send in place of the one answered by `res`, with the service to
    /// send it with, if one of them asks for a critical hint that request didn't have.
    pub(super) fn on_response<B>(&mut self, res: &Response<B>) -> Option<(S, Request<Body>)> {
        // Redirects the client followed, each with the URI that answered and its headers.
        let redirects = res
            .extensions()
            .get::<History>()
            .into_iter()
            .flatten()
            .map(|redirect| (origin(&redirect.previous), &redirect.headers));
        // Finding where the response came from copies its extensions, so only do it when needed.
        let answered = if res.headers().contains_key(ACCEPT_CH) {
            match response_uri(res) {
                Some(uri) => origin(&uri),
                None => Some(self.origin.clone()),
            }
        } else {
            None
        };

        let mut asked = Vec::new();
        for (origin, headers) in redirects.chain([(answered, res.headers())]) {
            let Some((origin, accepted)) = origin.zip(listed(headers, &ACCEPT_CH)) else {
                continue;
            };
            self.store.set(origin, &accepted);

            if self.needs_restart(headers, &accepted) {
                asked.extend(accepted.iter().cloned());
            }
        }
        if asked.is_empty() {
            return None;
        }

        // Restart with what the chain asked for, plus the request origin's own hints.
        let (service, mut req) = self.restart.take()?;
        if let Some(accepted) = self.store.get(&self.origin) {
            asked.extend(accepted.iter().cloned());
        }
        attach(&mut req, &self.hints, &asked, true);
        Some((service, req))
    }

    /// Returns whether a response asks for a critical hint the request didn't have.
    ///
    /// Like Chrome, only a hint in both `Critical-CH` and `Accept-CH` counts, and a request is
    /// only restarted once.
    fn needs_restart(&self, headers: &HeaderMap, accepted: &[HeaderName]) -> bool {
        let Some(critical) = listed(headers, &CRITICAL_CH) else {
            return false;
        };
        let Some((_, copy)) = &self.restart else {
            return false;
        };
        let sent = self.accepted.as_deref().unwrap_or_default();
        self.hints.iter().any(|(hint, _)| {
            critical.contains(hint)
                && accepted.contains(hint)
                && !sent.contains(hint)
                && !copy.headers().contains_key(hint)
        })
    }
}

/// Adds the hints `accepted` names to `req`, unless the request already sets them.
fn attach(req: &mut Request<Body>, hints: &Hints, accepted: &[HeaderName], restarted: bool) {
    let mut attached = false;
    for (hint, value) in hints.iter() {
        if accepted.contains(hint) && !req.headers().contains_key(hint) {
            req.headers_mut().insert(hint.clone(), value.clone());
            attached = true;
        }
    }

    // The client's own header order, if it has one, replaces this further down.
    if attached {
        wreq_proto::ext::on_preserve_header(req, HintOrder { restarted });
    }
}

/// Returns the URI a response came from, if the client recorded it.
///
/// The client only records it when it handles redirects.
fn response_uri<B>(res: &Response<B>) -> Option<Uri> {
    // `ResponseExt` reads it, but only from a response with a `wreq::Body`.
    let mut probe = Response::new(Body::default());
    *probe.extensions_mut() = res.extensions().clone();
    probe.uri().cloned()
}
