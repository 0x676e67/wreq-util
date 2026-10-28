use std::{
    net::IpAddr,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use lru::LruCache;
use wreq::{Uri, header::HeaderName};

/// The client hints an origin asked for.
pub(super) type Accepted = Arc<[HeaderName]>;

/// Remembers which client hints each origin asked for, like a browser profile does.
#[derive(Clone, Debug)]
pub(super) struct Store {
    /// Origins in the order they were first seen, or `None` when nothing is remembered.
    origins: Option<Arc<Mutex<LruCache<String, Accepted>>>>,
}

// ===== impl Store =====

impl Store {
    #[inline]
    pub(super) fn new(capacity: usize) -> Self {
        Store {
            origins: NonZeroUsize::new(capacity)
                .map(|cap| Arc::new(Mutex::new(LruCache::new(cap)))),
        }
    }

    /// Creates a store that never forgets an origin, for the hops of one request.
    #[inline]
    pub(super) fn unbounded() -> Self {
        Store {
            origins: Some(Arc::new(Mutex::new(LruCache::unbounded()))),
        }
    }

    /// Returns whether the store remembers any origin.
    #[inline]
    pub(super) fn remembers(&self) -> bool {
        self.origins.is_some()
    }

    /// Returns the client hints `origin` asked for.
    #[inline]
    pub(super) fn get(&self, origin: &str) -> Option<Accepted> {
        let origins = self.origins.as_ref()?;
        origins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .peek(origin)
            .cloned()
    }

    /// Replaces the client hints `origin` asked for. An empty list forgets the origin.
    pub(super) fn set(&self, origin: String, accepted: &Accepted) {
        let Some(origins) = &self.origins else {
            return;
        };
        let mut origins = origins.lock().unwrap_or_else(|e| e.into_inner());

        // `peek` leaves the order alone, so the origin seen first is forgotten first.
        if accepted.is_empty() {
            origins.pop(&origin);
        } else if let Some(current) = origins.peek_mut(&origin) {
            *current = accepted.clone();
        } else {
            origins.push(origin, accepted.clone());
        }
    }
}

/// Returns the origin of `uri` if browsers send client hints to it.
///
/// That is an `https` origin, or an `http` one on the local machine.
pub(super) fn origin(uri: &Uri) -> Option<String> {
    let scheme = uri.scheme_str()?;
    let host = uri.host()?.to_ascii_lowercase();
    let default_port = match scheme {
        "https" => 443,
        "http" if is_local(&host) => 80,
        _ => return None,
    };
    let port = uri.port_u16().unwrap_or(default_port);
    Some(format!("{scheme}://{host}:{port}"))
}

fn is_local(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    if host == "localhost" || host.ends_with(".localhost") {
        return true;
    }
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<IpAddr>()
        .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted(hints: &[&'static str]) -> Accepted {
        hints.iter().copied().map(HeaderName::from_static).collect()
    }

    #[test]
    fn origins_browsers_send_hints_to() {
        let origin = |uri: &str| origin(&uri.parse().unwrap());

        assert_eq!(
            origin("https://Example.com/a?b"),
            Some("https://example.com:443".to_owned())
        );
        assert_eq!(
            origin("https://example.com:8443/"),
            Some("https://example.com:8443".to_owned())
        );
        assert_eq!(
            origin("http://localhost:3000/"),
            Some("http://localhost:3000".to_owned())
        );
        assert_eq!(
            origin("http://app.localhost/"),
            Some("http://app.localhost:80".to_owned())
        );
        assert_eq!(
            origin("http://127.0.0.1:8080/"),
            Some("http://127.0.0.1:8080".to_owned())
        );
        assert_eq!(
            origin("http://[::1]:8080/"),
            Some("http://[::1]:8080".to_owned())
        );
        assert_eq!(origin("http://example.com/"), None);
        assert_eq!(origin("http://192.168.1.10/"), None);
        assert_eq!(origin("ws://localhost/"), None);
        assert_eq!(origin("/relative"), None);
    }

    #[test]
    fn replaces_and_forgets_origins() {
        let store = Store::new(8);
        assert_eq!(store.get("https://a:443"), None);

        store.set("https://a:443".to_owned(), &accepted(&["sec-ch-ua-arch"]));
        store.set("https://a:443".to_owned(), &accepted(&["sec-ch-ua-model"]));
        assert_eq!(
            store.get("https://a:443"),
            Some(accepted(&["sec-ch-ua-model"]))
        );

        store.set("https://a:443".to_owned(), &accepted(&[]));
        assert_eq!(store.get("https://a:443"), None);
    }

    #[test]
    fn forgets_the_oldest_origin_when_full() {
        let store = Store::new(2);
        let hints = accepted(&["sec-ch-ua-arch"]);

        store.set("https://a:443".to_owned(), &hints);
        store.set("https://b:443".to_owned(), &hints);
        store.set("https://a:443".to_owned(), &hints);
        store.set("https://c:443".to_owned(), &hints);

        assert_eq!(store.get("https://a:443"), None);
        assert_eq!(store.get("https://b:443"), Some(hints.clone()));
        assert_eq!(store.get("https://c:443"), Some(hints.clone()));

        let disabled = Store::new(0);
        disabled.set("https://a:443".to_owned(), &hints);
        assert_eq!(disabled.get("https://a:443"), None);
    }
}
