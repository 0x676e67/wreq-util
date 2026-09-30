#![cfg(not(target_arch = "wasm32"))]
mod support;

use std::future::Future;

use support::server;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use wreq::Client;
use wreq_util::{Emulation, Platform, Profile};

const CHROMIUM_HEADER_ORDER: &[&str] = &[
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-platform",
    "upgrade-insecure-requests",
    "user-agent",
    "accept",
    "sec-fetch-site",
    "sec-fetch-mode",
    "sec-fetch-user",
    "sec-fetch-dest",
    #[cfg(feature = "emulation-compression")]
    "accept-encoding",
    "accept-language",
    "priority",
];

const FIREFOX_HEADER_ORDER: &[&str] = &[
    "user-agent",
    "accept",
    "accept-language",
    #[cfg(feature = "emulation-compression")]
    "accept-encoding",
    "upgrade-insecure-requests",
    "sec-fetch-dest",
    "sec-fetch-mode",
    "sec-fetch-site",
    "sec-fetch-user",
    "te",
];

fn check_header_order<'a>(
    request: &'a [u8],
    stream: &'a mut TcpStream,
    expected: &'static [&'static str],
) -> Box<dyn Future<Output = ()> + Send + 'a> {
    Box::new(async move {
        let request = std::str::from_utf8(request)
            .expect("request should be valid UTF-8")
            .to_ascii_lowercase();
        let mut previous = None;

        for name in expected {
            let marker = format!("\r\n{name}:");
            let position = request
                .find(&marker)
                .unwrap_or_else(|| panic!("missing `{name}` header:\n{request}"));

            if let Some(previous) = previous {
                assert!(
                    position > previous,
                    "`{name}` header is out of order:\n{request}"
                );
            }

            previous = Some(position);
        }

        assert!(request.contains("\r\nupgrade-insecure-requests: 1\r\n"));
        assert!(request.contains("\r\nsec-fetch-user: ?1\r\n"));

        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
            .await
            .expect("response");
    })
}

async fn assert_emulation_headers(profile: Profile, expected: &'static [&'static str]) {
    let server = server::low_level_with_response(move |request, stream| {
        check_header_order(request, stream, expected)
    });
    let response = Client::builder()
        .emulation(profile)
        .build()
        .expect("client")
        .get(format!("http://{}/headers", server.addr()))
        .send()
        .await
        .expect("request");

    assert_eq!(response.status(), wreq::StatusCode::OK);
}

#[tokio::test]
async fn test_client_emulation_device() {
    let server = server::http(move |req| async move {
        let header = |name| {
            req.headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
        };

        assert_eq!(
            header("user-agent"),
            Some(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36"
            )
        );
        assert_eq!(
            header("sec-ch-ua"),
            Some(r#""Not=A?Brand";v="99", "Google Chrome";v="151", "Chromium";v="151""#)
        );
        assert_eq!(header("sec-ch-ua-mobile"), Some("?0"));
        assert_eq!(header("sec-ch-ua-platform"), Some("\"Linux\""));
        http::Response::default()
    });

    let url = format!("http://{}/ua", server.addr());
    let res = Client::builder()
        .emulation(
            Emulation::builder()
                .profile(Emulation::Chrome151)
                .platform(Platform::Linux)
                .http2(true)
                .build(),
        )
        .build()
        .expect("Unable to build client")
        .get(&url)
        .send()
        .await
        .expect("request");

    assert_eq!(res.status(), wreq::StatusCode::OK);
}

#[tokio::test]
async fn test_chrome_default_header_order() {
    assert_emulation_headers(Emulation::Chrome133, CHROMIUM_HEADER_ORDER).await;
}

#[tokio::test]
async fn test_firefox_default_header_order() {
    assert_emulation_headers(Emulation::Firefox109, FIREFOX_HEADER_ORDER).await;
}

#[tokio::test]
async fn test_firefox147_and_later_header_order() {
    const HEADER_ORDER: &[&str] = &[
        "user-agent",
        "accept",
        "accept-language",
        #[cfg(feature = "emulation-compression")]
        "accept-encoding",
        "upgrade-insecure-requests",
        "sec-fetch-dest",
        "sec-fetch-mode",
        "sec-fetch-site",
        "sec-fetch-user",
        "priority",
        "te",
    ];
    for profile in [
        Emulation::Firefox147,
        Emulation::Firefox148,
        Emulation::Firefox149,
        Emulation::Firefox150,
        Emulation::Firefox151,
        Emulation::Firefox152,
    ] {
        assert_emulation_headers(profile, HEADER_ORDER).await;
    }
}

#[tokio::test]
async fn test_opera_default_header_order() {
    assert_emulation_headers(Emulation::Opera116, CHROMIUM_HEADER_ORDER).await;
}

async fn assert_chrome152_headers(
    platform: Platform,
    expected_platform: &'static str,
    expected_user_agent: &'static str,
) {
    let server = server::http(move |req| async move {
        assert_eq!(
            req.headers().get("sec-ch-ua").unwrap(),
            r#""Chromium";v="152", "Not?A_Brand";v="24", "Google Chrome";v="152""#
        );
        assert_eq!(
            req.headers().get("sec-ch-ua-platform").unwrap(),
            expected_platform
        );
        assert_eq!(
            req.headers().get("user-agent").unwrap(),
            expected_user_agent
        );
        http::Response::default()
    });

    let url = format!("http://{}/ua", server.addr());
    let res = Client::builder()
        .emulation(
            Emulation::builder()
                .profile(Emulation::Chrome152)
                .platform(platform)
                .build(),
        )
        .build()
        .expect("Unable to build client")
        .get(&url)
        .send()
        .await
        .expect("request");

    assert_eq!(res.status(), wreq::StatusCode::OK);
}

#[tokio::test]
async fn test_chrome152_windows_and_macos_headers() {
    assert_chrome152_headers(
        Platform::Windows,
        "\"Windows\"",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36",
    )
    .await;
    assert_chrome152_headers(
        Platform::MacOS,
        "\"macOS\"",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36",
    )
    .await;
}

/// Starts a server that echoes the received `user-agent` and `sec-ch-ua`.
fn echo_ua_server() -> server::Server {
    server::http(move |req| async move {
        let header = |name| {
            req.headers()
                .get(name)
                .map(|value| value.to_str().unwrap().to_owned())
                .unwrap_or_default()
        };
        let body = format!("{}\n{}", header("user-agent"), header("sec-ch-ua"));
        http::Response::new(body.into())
    })
}

/// Returns the `user-agent` and `sec-ch-ua` a profile sends on a platform.
async fn sent_ua(
    server: &server::Server,
    profile: Profile,
    platform: Platform,
) -> (String, String) {
    let text = Client::builder()
        .emulation(
            Emulation::builder()
                .profile(profile)
                .platform(platform)
                .build(),
        )
        .build()
        .expect("client")
        .get(format!("http://{}/", server.addr()))
        .send()
        .await
        .expect("request")
        .text()
        .await
        .expect("body");
    let (ua, sec_ch_ua) = text.split_once('\n').expect("echo body");
    (ua.to_owned(), sec_ch_ua.to_owned())
}

const DESKTOP: [Platform; 3] = [Platform::MacOS, Platform::Windows, Platform::Linux];

#[tokio::test]
async fn test_edge_user_agent_has_reduced_edg_token() {
    let server = echo_ua_server();
    for (profile, major) in [
        (Emulation::Edge134, 134),
        (Emulation::Edge135, 135),
        (Emulation::Edge136, 136),
        (Emulation::Edge137, 137),
        (Emulation::Edge138, 138),
        (Emulation::Edge139, 139),
        (Emulation::Edge140, 140),
        (Emulation::Edge141, 141),
        (Emulation::Edge146, 146),
        (Emulation::Edge147, 147),
    ] {
        for platform in DESKTOP {
            let (ua, _) = sent_ua(&server, profile, platform).await;
            let tail = format!(" Safari/537.36 Edg/{major}.0.0.0");
            assert!(ua.ends_with(&tail), "{profile:?} {platform:?}: {ua}");
        }
        if major <= 141 {
            let (ua, _) = sent_ua(&server, profile, Platform::Android).await;
            let tail = format!(" Mobile Safari/537.36 EdgA/{major}.0.0.0");
            assert!(ua.ends_with(&tail), "{profile:?} Android: {ua}");
        }
    }
}

#[tokio::test]
async fn test_firefox139_rv_matches_version() {
    let server = echo_ua_server();
    for platform in DESKTOP {
        let (ua, _) = sent_ua(&server, Emulation::Firefox139, platform).await;
        assert!(
            ua.contains("rv:139.0) Gecko/20100101 Firefox/139.0"),
            "{platform:?}: {ua}"
        );
    }
}

#[tokio::test]
async fn test_firefox152_platform_headers() {
    for (platform, expected_ua) in [
        (
            Platform::Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:152.0) Gecko/20100101 Firefox/152.0",
        ),
        (
            Platform::MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:152.0) Gecko/20100101 Firefox/152.0",
        ),
        (
            Platform::Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0",
        ),
        (
            Platform::Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:152.0) Gecko/152.0 Firefox/152.0",
        ),
        (
            Platform::IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/152.0 Mobile/15E148 Safari/605.1.15",
        ),
    ] {
        let server = server::http(move |req| async move {
            assert_eq!(req.headers().get("user-agent").unwrap(), expected_ua);
            assert_eq!(
                req.headers().get("accept-language").unwrap(),
                "en-US,en;q=0.9"
            );
            #[cfg(feature = "emulation-compression")]
            assert_eq!(
                req.headers().get("accept-encoding").unwrap(),
                "gzip, deflate, br, zstd"
            );
            #[cfg(not(feature = "emulation-compression"))]
            assert!(!req.headers().contains_key("accept-encoding"));
            http::Response::default()
        });
        let response = Client::builder()
            .no_proxy()
            .emulation(
                Emulation::builder()
                    .profile(Emulation::Firefox152)
                    .platform(platform)
                    .build(),
            )
            .build()
            .unwrap()
            .get(format!("http://{}/", server.addr()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), wreq::StatusCode::OK);
    }
}

#[tokio::test]
async fn test_safari17_2_1_version_token() {
    let server = echo_ua_server();
    let (ua, _) = sent_ua(&server, Emulation::Safari17_2_1, Platform::MacOS).await;
    assert!(ua.contains("Version/17.2.1 Safari/605.1.15"), "{ua}");
}

#[tokio::test]
async fn test_user_agent_literals_are_well_formed() {
    let server = echo_ua_server();
    for &profile in Profile::VARIANTS {
        for &platform in Platform::VARIANTS {
            let (ua, sec_ch_ua) = sent_ua(&server, profile, platform).await;
            assert!(!ua.contains("(Linux: "), "{profile:?} {platform:?}: {ua}");
            assert!(
                sec_ch_ua.is_empty()
                    || (sec_ch_ua.ends_with('"') && sec_ch_ua.matches('"').count() % 2 == 0),
                "{profile:?} {platform:?}: unbalanced sec-ch-ua {sec_ch_ua}"
            );
        }
    }
}
