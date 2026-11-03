use super::*;

pub fn header_initializer(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_chrome_sec_ch_ua!(
        headers,
        sec_ch_ua,
        emulation_os.platform(),
        emulation_os.is_mobile()
    );
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_chrome_ua!(headers, ua);
    header_chrome_accept!(headers);
    header_chrome_sec_fetch!(headers);
    header_chrome_accept_encoding!(headers);
    headers
}

pub fn header_initializer_with_zstd(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_chrome_sec_ch_ua!(
        headers,
        sec_ch_ua,
        emulation_os.platform(),
        emulation_os.is_mobile()
    );
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_chrome_ua!(headers, ua);
    header_chrome_accept!(headers);
    header_chrome_sec_fetch!(headers);
    header_chrome_accept_encoding!(zstd, headers);
    headers
}

pub fn header_initializer_with_zstd_priority(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_chrome_sec_ch_ua!(
        headers,
        sec_ch_ua,
        emulation_os.platform(),
        emulation_os.is_mobile()
    );
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_chrome_ua!(headers, ua);
    header_chrome_accept!(headers);
    header_chrome_sec_fetch!(headers);
    header_chrome_accept_encoding!(zstd, headers);
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=0, i"),
    );
    headers
}

// Windows Chrome 155 navigation headers; TLS/H2 parameters are shared with 154.
pub fn header_initializer_chrome155(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
) -> HeaderMap {
    let mut headers = header_initializer_with_zstd_priority(sec_ch_ua, ua, emulation_os);
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/jxl,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
    headers
}

pub fn header_initializer_chrome152(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_chrome_sec_ch_ua!(
        headers,
        sec_ch_ua,
        emulation_os.platform(),
        emulation_os.is_mobile()
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    header_chrome_ua!(headers, ua);
    header_chrome_accept!(headers);
    header_chrome_sec_fetch!(headers);
    header_chrome_accept_encoding!(zstd_only, headers);
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=0, i"),
    );
    headers
}
