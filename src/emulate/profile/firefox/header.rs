use super::*;

pub fn header_initializer(ua: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_firefox_ua!(headers, ua);
    header_firefox_accept!(headers);
    headers.insert(
        UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );
    header_firefox_sec_fetch!(headers);
    headers.insert(
        TE,
        HeaderValue::from_static("trailers"),
    );
    headers
}

pub fn header_initializer_with_zstd(ua: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_firefox_ua!(headers, ua);
    header_firefox_accept!(zstd, headers);
    headers.insert(
        UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );
    header_firefox_sec_fetch!(headers);
    headers.insert(
        PRIORITY,
        HeaderValue::from_static("u=0, i"),
    );
    headers.insert(
        TE,
        HeaderValue::from_static("trailers"),
    );
    headers
}
