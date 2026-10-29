use std::{
    mem,
    ops::ControlFlow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use http::{Request, Response};
use wreq::{
    header::{HeaderMap, HeaderName},
    redirect::Policy,
};

use super::{
    hints::Hints,
    order::HintOrder,
    parse::{ACCEPT_CH, CRITICAL_CH, listed},
    store::{Accepted, Store, origin},
};

/// Sets the redirect callbacks that pick the client hints of every hop.
///
/// Without them, the hints picked for the first origin follow a request across redirects. With
/// them, each hop gets the hints its own origin accepts, `Accept-CH` of every redirect is
/// remembered as it arrives, and a redirect that asks for a critical hint restarts the request,
/// like Chrome does.
///
/// Replaces the request and response callbacks of `policy`.
///
/// # Example
///
/// ```no_run
/// use wreq::{Client, redirect::Policy};
/// use wreq_util::{
///     Emulation,
///     tower::client_hints::{self, ClientHintsLayer, Hints},
/// };
///
/// let client = Client::builder()
///     .emulation(Emulation::Chrome149)
///     .layer(ClientHintsLayer::new(Hints::from_emulation(Emulation::Chrome149)))
///     .redirect(client_hints::redirect(Policy::default()))
///     .build()?;
/// # Ok::<(), wreq::Error>(())
/// ```
pub fn redirect(policy: Policy) -> Policy {
    policy.on_request(on_request).on_response(on_response)
}

/// The client hints of a request, carried across its redirect hops.
#[derive(Clone)]
pub(super) struct Hops {
    hints: Arc<Hints>,
    store: Store,
    /// Whether the request can still be restarted.
    restartable: bool,
    /// The hints added to the current hop.
    attached: Vec<HeaderName>,
    report: Arc<Report>,
}

/// What the redirect callbacks tell the response future of a request.
#[derive(Default)]
pub(super) struct Report {
    /// Whether the callbacks handled the hops of the request.
    handled: AtomicBool,
    /// The origin of the hop that asked for a critical hint it didn't have, with the hints it
    /// accepts.
    critical: Mutex<Option<(String, Accepted)>>,
}

// ===== impl Hops =====

impl Hops {
    #[inline]
    pub(super) fn new(hints: Arc<Hints>, store: Store, restartable: bool) -> Self {
        Hops {
            hints,
            store,
            restartable,
            attached: Vec::new(),
            report: Arc::default(),
        }
    }

    /// Adds the hints `accepted` names to `req`, and hands the request to the redirect callbacks.
    ///
    /// Returns what the callbacks report about the request.
    pub(super) fn attach<B>(
        mut self,
        req: &mut Request<B>,
        accepted: &[HeaderName],
        restarted: bool,
    ) -> Arc<Report> {
        self.add(req.headers_mut(), accepted);
        let report = self.report.clone();
        req.extensions_mut().insert(self);

        // Hops can't tell the client's own order from this one, so it is set once here and
        // carried across hops. An `orig_headers` order set on the client replaces it.
        wreq_proto::ext::on_preserve_header(req, HintOrder { restarted });
        report
    }

    /// Adds the hints `accepted` names, unless the request already sets them.
    fn add(&mut self, headers: &mut HeaderMap, accepted: &[HeaderName]) {
        for (hint, value) in self.hints.iter() {
            if accepted.contains(hint) && !headers.contains_key(hint) {
                headers.insert(hint.clone(), value.clone());
                self.attached.push(hint.clone());
            }
        }
    }
}

// ===== impl Report =====

impl Report {
    /// Returns whether the callbacks handled the hops of the request.
    #[inline]
    pub(super) fn handled(&self) -> bool {
        self.handled.load(Ordering::Relaxed)
    }

    /// Takes the hop that asked for a critical hint it didn't have.
    #[inline]
    pub(super) fn take_critical(&self) -> Option<(String, Accepted)> {
        self.critical
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }
}

/// Swaps the hints of a hop for the ones its origin accepts.
fn on_request(req: &mut Request<()>) {
    let Some(mut hops) = req.extensions_mut().remove::<Hops>() else {
        return;
    };
    hops.report.handled.store(true, Ordering::Relaxed);

    for hint in mem::take(&mut hops.attached) {
        req.headers_mut().remove(hint);
    }
    let accepted = origin(req.uri())
        .and_then(|origin| hops.store.get(&origin))
        .unwrap_or_default();
    hops.add(req.headers_mut(), &accepted);
    req.extensions_mut().insert(hops);
}

/// Remembers the hints a hop's origin accepts, and stops at a hop that needs a restart.
///
/// Like Chrome, a redirect that asks for a critical hint isn't followed — the request restarts
/// from its first URL instead.
fn on_response(req: &Request<()>, res: &Response<()>) -> ControlFlow<()> {
    let Some(hops) = req.extensions().get::<Hops>() else {
        return ControlFlow::Continue(());
    };
    let Some((hop, accepted)) = origin(req.uri()).zip(listed(res.headers(), &ACCEPT_CH)) else {
        return ControlFlow::Continue(());
    };
    hops.store.set(hop.clone(), &accepted);

    // Like Chrome, only a hint in both `Critical-CH` and `Accept-CH` counts.
    let Some(critical) = listed(res.headers(), &CRITICAL_CH) else {
        return ControlFlow::Continue(());
    };
    let missing = hops.hints.iter().any(|(hint, _)| {
        critical.contains(hint) && accepted.contains(hint) && !req.headers().contains_key(hint)
    });
    if !(missing && hops.restartable) {
        return ControlFlow::Continue(());
    }

    hops.report
        .critical
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .replace((hop, accepted));
    ControlFlow::Break(())
}
