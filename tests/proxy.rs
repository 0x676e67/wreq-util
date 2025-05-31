#![cfg(not(target_arch = "wasm32"))]
mod support;

use rquest::{Client, Proxy};
use rquest_util::Emulation;
use support::server;

#[tokio::test]
async fn test_http_proxy() {
    let server = server::http(move |req| async move {
        // For HTTP proxy, we should see the full URL in the request
        assert!(req.uri().to_string().starts_with("http://example.com"));
        http::Response::default()
    });

    let proxy_url = format!("http://{}", server.addr());
    let proxy = Proxy::all(proxy_url).expect("Failed to create proxy config");

    let client = Client::builder()
        .emulation(Emulation::Chrome136)
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    let res = client
        .get("http://example.com")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), rquest::StatusCode::OK);
}

#[tokio::test]
async fn test_socks5_proxy() {
    let server = server::http(move |req| async move {
        // For SOCKS5 proxy, we should see the target host in the request
        assert!(req.uri().to_string().starts_with("http://example.com"));
        http::Response::default()
    });

    let proxy_url = format!("socks5://{}", server.addr());
    let proxy = Proxy::all(proxy_url).expect("Failed to create proxy config");

    let client = Client::builder()
        .emulation(Emulation::Chrome136)
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    let res = client
        .get("http://example.com")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), rquest::StatusCode::OK);
}

#[tokio::test]
async fn test_proxy_with_auth() {
    let server = server::http(move |req| async move {
        // For HTTP proxy with auth, we should see the full URL and auth header
        assert!(req.uri().to_string().starts_with("http://example.com"));
        assert!(req.headers().contains_key("proxy-authorization"));
        http::Response::default()
    });

    let proxy_url = format!("http://user:pass@{}", server.addr());
    let proxy = Proxy::all(proxy_url).expect("Failed to create proxy config");

    let client = Client::builder()
        .emulation(Emulation::Chrome136)
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    let res = client
        .get("http://example.com")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), rquest::StatusCode::OK);
}

#[tokio::test]
async fn test_socks5_proxy_with_auth() {
    let server = server::http(move |req| async move {
        // For SOCKS5 proxy with auth, we should see the target host and auth header
        assert!(req.uri().to_string().starts_with("http://example.com"));
        assert!(req.headers().contains_key("proxy-authorization"));
        http::Response::default()
    });

    let proxy_url = format!("socks5://user:pass@{}", server.addr());
    let proxy = Proxy::all(proxy_url).expect("Failed to create proxy config");

    let client = Client::builder()
        .emulation(Emulation::Chrome136)
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    let res = client
        .get("http://example.com")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), rquest::StatusCode::OK);
}

#[tokio::test]
async fn test_real_socks5_proxy() {
    let proxy_url = std::env::var("RQUEST_PROXY_URL").expect("RQUEST_PROXY_URL environment variable must be set");
    let proxy = Proxy::all(proxy_url).expect("Failed to create proxy config");

    let client = Client::builder()
        .emulation(Emulation::Chrome136)
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    // Test with a real website that's commonly used for IP testing
    let res = client
        .get("https://api.ipify.org?format=json")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), rquest::StatusCode::OK);
    
    // Parse the response to verify we got a valid IP
    let body = res.text().await.expect("Failed to get response body");
    println!("Response from proxy: {}", body);
    
    // Verify the response contains an IP address
    assert!(body.contains("ip"));
} 