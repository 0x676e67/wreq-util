use std::sync::Arc;

use tower::Layer;

use super::{hints::Hints, service::ClientHints, store::Store};

/// Default for [`ClientHintsLayer::max_origins`].
const DEFAULT_MAX_ORIGINS: usize = 1024;

/// A Tower [`Layer`] that sends the client hints an origin accepts, and restarts a request for
/// the critical ones it lacks.
///
/// See [`crate::tower::client_hints`] for more details.
///
/// Clones of the layer, and the services they build, share one origin store.
#[derive(Clone, Debug)]
pub struct ClientHintsLayer {
    hints: Option<Arc<Hints>>,
    store: Store,
}

// ===== impl ClientHintsLayer =====

impl ClientHintsLayer {
    /// Creates a new [`ClientHintsLayer`] with the given default [`Hints`].
    ///
    /// With `None`, only requests that carry their own `Sec-CH-UA` get hints.
    #[inline]
    pub fn new(hints: impl Into<Option<Hints>>) -> Self {
        ClientHintsLayer {
            hints: hints.into().map(Arc::new),
            store: Store::new(DEFAULT_MAX_ORIGINS),
        }
    }

    /// Sets the maximum number of origins to remember.
    ///
    /// Default is `1024`. The oldest origin is forgotten first, and `0` disables remembering.
    /// Call it before cloning the layer, since it starts a new origin store.
    #[inline]
    pub fn max_origins(mut self, max_origins: usize) -> Self {
        self.store = Store::new(max_origins);
        self
    }
}

impl<S> Layer<S> for ClientHintsLayer {
    type Service = ClientHints<S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        ClientHints::new(inner, self.hints.clone(), self.store.clone())
    }
}
