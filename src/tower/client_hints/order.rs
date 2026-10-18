use wreq::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue};
use wreq_proto::ext::OnPreserveHeaderCallback;

/// The order Chrome writes client hint headers in, from `AddRequestClientHintsHeaders` and
/// checked against Chrome 154:
/// <https://source.chromium.org/chromium/chromium/src/+/main:content/browser/client_hints/client_hints.cc>
const ORDER: &[&str] = &[
    "device-memory",
    "sec-ch-device-memory",
    "dpr",
    "sec-ch-dpr",
    "viewport-width",
    "sec-ch-viewport-width",
    "sec-ch-viewport-height",
    "rtt",
    "downlink",
    "ect",
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-full-version",
    "sec-ch-ua-arch",
    "sec-ch-ua-platform",
    "sec-ch-ua-platform-version",
    "sec-ch-ua-model",
    "sec-ch-ua-bitness",
    "sec-ch-ua-wow64",
    "sec-ch-ua-full-version-list",
    "sec-ch-ua-form-factors",
    "sec-ch-prefers-color-scheme",
    "sec-ch-prefers-reduced-motion",
    "sec-ch-prefers-reduced-transparency",
    "save-data",
];

/// Moves the client hints of a request to where Chrome writes them.
///
/// The client merges its default headers in after this layer, which leaves the hints after all
/// of them.
pub(super) struct HintOrder {
    /// Whether the request is one restarted for `Critical-CH`.
    pub(super) restarted: bool,
}

// ===== impl HintOrder =====

impl OnPreserveHeaderCallback for HintOrder {
    fn call(&self, headers: &mut HeaderMap) {
        reorder(headers, self.restarted);
    }

    fn call_visit(
        &self,
        headers: &mut HeaderMap,
        dst: &mut dyn FnMut(&dyn AsRef<[u8]>, &HeaderValue),
    ) {
        reorder(headers, self.restarted);
        for (name, value) in headers.iter() {
            dst(name, value);
        }
    }
}

/// Returns the position of `name` among the headers Chrome writes as client hints.
pub(super) fn rank(name: &HeaderName) -> Option<usize> {
    ORDER.iter().position(|hint| *hint == name.as_str())
}

/// Groups every client hint header in Chrome's order.
///
/// Chrome writes them as one block where `Sec-CH-UA` is. When it restarts a request it takes that
/// block out and adds it again, which leaves it right after `Accept`.
fn reorder(headers: &mut HeaderMap, restarted: bool) {
    let mut hints = Vec::new();
    let mut others = Vec::with_capacity(headers.len());
    // The number of other headers written before the hints.
    let mut first_hint = None;
    let mut after_accept = None;

    for (name, value) in drain(headers) {
        match rank(&name) {
            Some(rank) => {
                first_hint.get_or_insert(others.len());
                hints.push((rank, name, value));
            }
            None => {
                if name == ACCEPT {
                    after_accept = Some(others.len() + 1);
                }
                others.push((name, value));
            }
        }
    }
    hints.sort_by_key(|(rank, ..)| *rank);

    let position = after_accept
        .filter(|_| restarted)
        .or(first_hint)
        .unwrap_or(others.len());

    let hints = hints.into_iter().map(|(_, name, value)| (name, value));
    let after = others.split_off(position);
    for (name, value) in others.into_iter().chain(hints).chain(after) {
        headers.append(name, value);
    }
}

/// Takes the headers out of a map, in order and each with its name.
fn drain(headers: &mut HeaderMap) -> Vec<(HeaderName, HeaderValue)> {
    let mut drained = Vec::with_capacity(headers.len());
    let mut prev_name: Option<HeaderName> = None;
    for (name, value) in headers.drain() {
        // A header with several values only yields its name with the first one.
        match (name, &prev_name) {
            (Some(name), _) => {
                prev_name.replace(name.clone());
                drained.push((name, value));
            }
            (None, Some(prev_name)) => drained.push((prev_name.clone(), value)),
            _ => {}
        }
    }
    drained
}

#[cfg(test)]
mod tests {
    use super::*;

    // The headers of a navigation by Chrome 154 that are set before the request reaches the
    // network stack, in the order it sent them.
    const DEFAULTS: &[&str] = &[
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
        "upgrade-insecure-requests",
        "user-agent",
        "accept",
        "sec-fetch-site",
        "sec-fetch-mode",
        "sec-fetch-dest",
        "accept-encoding",
        "accept-language",
    ];

    fn request(extra: &[&'static str]) -> HeaderMap {
        DEFAULTS
            .iter()
            .chain(extra)
            .map(|name| (HeaderName::from_static(name), HeaderValue::from_static("x")))
            .collect()
    }

    fn names(headers: &HeaderMap) -> Vec<&str> {
        headers.keys().map(HeaderName::as_str).collect()
    }

    #[test]
    fn writes_hints_where_chrome_does() {
        let mut headers = request(&[
            "sec-ch-ua-form-factors",
            "sec-ch-ua-arch",
            "sec-ch-prefers-color-scheme",
            "sec-ch-ua-full-version-list",
            "sec-ch-ua-model",
            "sec-ch-ua-full-version",
            "sec-ch-ua-wow64",
            "sec-ch-ua-bitness",
            "sec-ch-ua-platform-version",
            "sec-ch-dpr",
        ]);
        reorder(&mut headers, false);

        assert_eq!(
            names(&headers),
            [
                "sec-ch-dpr",
                "sec-ch-ua",
                "sec-ch-ua-mobile",
                "sec-ch-ua-full-version",
                "sec-ch-ua-arch",
                "sec-ch-ua-platform",
                "sec-ch-ua-platform-version",
                "sec-ch-ua-model",
                "sec-ch-ua-bitness",
                "sec-ch-ua-wow64",
                "sec-ch-ua-full-version-list",
                "sec-ch-ua-form-factors",
                "sec-ch-prefers-color-scheme",
                "upgrade-insecure-requests",
                "user-agent",
                "accept",
                "sec-fetch-site",
                "sec-fetch-mode",
                "sec-fetch-dest",
                "accept-encoding",
                "accept-language",
            ]
        );
    }

    #[test]
    fn writes_hints_after_accept_when_restarted() {
        let mut headers = request(&["sec-ch-ua-model", "sec-ch-ua-arch"]);
        reorder(&mut headers, true);

        assert_eq!(
            names(&headers),
            [
                "upgrade-insecure-requests",
                "user-agent",
                "accept",
                "sec-ch-ua",
                "sec-ch-ua-mobile",
                "sec-ch-ua-arch",
                "sec-ch-ua-platform",
                "sec-ch-ua-model",
                "sec-fetch-site",
                "sec-fetch-mode",
                "sec-fetch-dest",
                "accept-encoding",
                "accept-language",
            ]
        );
    }

    #[test]
    fn keeps_other_headers_and_repeated_values() {
        let mut headers = HeaderMap::new();
        headers.append("content-length", HeaderValue::from_static("3"));
        headers.append("x-first", HeaderValue::from_static("1"));
        headers.append("x-first", HeaderValue::from_static("2"));
        headers.append("sec-ch-ua", HeaderValue::from_static("x"));
        headers.append("user-agent", HeaderValue::from_static("x"));
        headers.append("x-last", HeaderValue::from_static("x"));
        headers.append("sec-ch-ua-arch", HeaderValue::from_static("x"));
        reorder(&mut headers, true);

        assert_eq!(
            names(&headers),
            [
                "content-length",
                "x-first",
                "sec-ch-ua",
                "sec-ch-ua-arch",
                "user-agent",
                "x-last",
            ]
        );
        let first: Vec<_> = headers.get_all("x-first").iter().collect();
        assert_eq!(first, ["1", "2"]);
    }

    #[test]
    fn appends_hints_when_nothing_anchors_them() {
        let mut headers = HeaderMap::new();
        headers.append("user-agent", HeaderValue::from_static("x"));
        reorder(&mut headers, false);
        assert_eq!(names(&headers), ["user-agent"]);

        let mut visited = Vec::new();
        headers.append("sec-ch-ua-model", HeaderValue::from_static("m"));
        headers.append("sec-ch-ua-arch", HeaderValue::from_static("a"));
        HintOrder { restarted: false }.call_visit(&mut headers, &mut |name, value| {
            visited.push((name.as_ref().to_vec(), value.clone()));
        });
        assert_eq!(
            visited,
            [
                (b"user-agent".to_vec(), HeaderValue::from_static("x")),
                (b"sec-ch-ua-arch".to_vec(), HeaderValue::from_static("a")),
                (b"sec-ch-ua-model".to_vec(), HeaderValue::from_static("m")),
            ]
        );
    }
}
