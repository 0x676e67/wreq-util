use super::*;

pub fn header_initializer(ua: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_firefox_ua!(headers, ua);
    header_firefox_accept!(headers);
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_firefox_sec_fetch!(headers);
    headers.insert(
        HeaderName::from_static("te"),
        HeaderValue::from_static("trailers"),
    );
    headers
}

pub fn header_initializer_with_zstd(ua: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_firefox_ua!(headers, ua);
    header_firefox_accept!(zstd, headers);
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_firefox_sec_fetch!(headers);
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=0, i"),
    );
    headers.insert(
        HeaderName::from_static("te"),
        HeaderValue::from_static("trailers"),
    );
    headers
}

/// Firefox 147 and later. Identical to [`header_initializer_with_zstd`] apart from the
/// `Accept-Language` q-value, which Firefox changed in 147 (Bugzilla 2000765).
pub fn header_initializer_with_zstd_q09(ua: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_firefox_ua!(headers, ua);
    header_firefox_accept!(zstd_q09, headers);
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_firefox_sec_fetch!(headers);
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=0, i"),
    );
    headers.insert(
        HeaderName::from_static("te"),
        HeaderValue::from_static("trailers"),
    );
    headers
}
