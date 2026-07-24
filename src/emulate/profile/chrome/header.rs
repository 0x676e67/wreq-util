use super::*;
use crate::emulate::wtf_hash;

pub fn header_initializer(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
    strategy: Strategy,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    match strategy {
        Strategy::Navigate => {
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
        }
        Strategy::Fetch => {
            let mobile = if emulation_os.is_mobile() { "?1" } else { "?0" };
            let platform = emulation_os.platform();
            for name in wtf_hash::header_order(&[
                "sec-ch-ua",
                "sec-ch-ua-mobile",
                "sec-ch-ua-platform",
                "user-agent",
            ]) {
                match name {
                    "sec-ch-ua" => {
                        headers.insert("sec-ch-ua", HeaderValue::from_static(sec_ch_ua));
                    }
                    "sec-ch-ua-mobile" => {
                        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static(mobile));
                    }
                    "sec-ch-ua-platform" => {
                        headers.insert(
                            "sec-ch-ua-platform",
                            HeaderValue::from_static(platform),
                        );
                    }
                    "user-agent" => {
                        header_chrome_ua!(headers, ua);
                    }
                    _ => {}
                }
            }
            headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
            headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
            header_chrome_accept_encoding!(headers);
        }
    }
    headers
}

pub fn header_initializer_with_zstd(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
    strategy: Strategy,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    match strategy {
        Strategy::Navigate => {
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
        }
        Strategy::Fetch => {
            let mobile = if emulation_os.is_mobile() { "?1" } else { "?0" };
            let platform = emulation_os.platform();
            for name in wtf_hash::header_order(&[
                "sec-ch-ua",
                "sec-ch-ua-mobile",
                "sec-ch-ua-platform",
                "user-agent",
            ]) {
                match name {
                    "sec-ch-ua" => {
                        headers.insert("sec-ch-ua", HeaderValue::from_static(sec_ch_ua));
                    }
                    "sec-ch-ua-mobile" => {
                        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static(mobile));
                    }
                    "sec-ch-ua-platform" => {
                        headers.insert(
                            "sec-ch-ua-platform",
                            HeaderValue::from_static(platform),
                        );
                    }
                    "user-agent" => {
                        header_chrome_ua!(headers, ua);
                    }
                    _ => {}
                }
            }
            headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
            headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
            header_chrome_accept_encoding!(zstd, headers);
        }
    }
    headers
}

pub fn header_initializer_with_zstd_priority(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: Platform,
    strategy: Strategy,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    match strategy {
        Strategy::Navigate => {
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
        }
        Strategy::Fetch => {
            let mobile = if emulation_os.is_mobile() { "?1" } else { "?0" };
            let platform = emulation_os.platform();
            for name in wtf_hash::header_order(&[
                "sec-ch-ua",
                "sec-ch-ua-mobile",
                "sec-ch-ua-platform",
                "user-agent",
            ]) {
                match name {
                    "sec-ch-ua" => {
                        headers.insert("sec-ch-ua", HeaderValue::from_static(sec_ch_ua));
                    }
                    "sec-ch-ua-mobile" => {
                        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static(mobile));
                    }
                    "sec-ch-ua-platform" => {
                        headers.insert(
                            "sec-ch-ua-platform",
                            HeaderValue::from_static(platform),
                        );
                    }
                    "user-agent" => {
                        header_chrome_ua!(headers, ua);
                    }
                    _ => {}
                }
            }
            headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
            headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
            header_chrome_accept_encoding!(zstd, headers);
            headers.insert(
                HeaderName::from_static("priority"),
                HeaderValue::from_static("u=1, i"),
            );
        }
    }
    headers
}
