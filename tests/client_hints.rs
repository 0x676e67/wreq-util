#![cfg(not(target_arch = "wasm32"))]
mod support;

use std::sync::{Arc, Mutex};

use support::server;
use wreq::{
    Client, ClientBuilder, Method, StatusCode, Version,
    header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    redirect,
};
use wreq_util::{
    Emulation, Platform, Profile,
    tower::client_hints::{ClientHintsLayer, Hints},
};

// What a Cloudflare challenge that depends on client hints sends, legacy names included.
const CHALLENGE_HINTS: &str = "Sec-CH-UA-Bitness, Sec-CH-UA-Arch, Sec-CH-UA-Full-Version, \
     Sec-CH-UA-Mobile, Sec-CH-UA-Model, Sec-CH-UA-Platform-Version, Sec-CH-UA-Full-Version-List, \
     Sec-CH-UA-Platform, Sec-CH-UA, UA-Bitness, UA-Arch, UA-Full-Version, UA-Mobile, UA-Model, \
     UA-Platform-Version, UA-Platform, UA";

const ALL_UA_HINTS: &str = "Sec-CH-UA-Arch, Sec-CH-UA-Bitness, Sec-CH-UA-Form-Factors, \
     Sec-CH-UA-Full-Version, Sec-CH-UA-Full-Version-List, Sec-CH-UA-Model, \
     Sec-CH-UA-Platform-Version, Sec-CH-UA-WoW64";

struct Recorded {
    method: Method,
    path: String,
    headers: HeaderMap,
}

impl Recorded {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    /// The high-entropy client hints of the request, in the order they arrived.
    fn hints(&self) -> Vec<&str> {
        self.headers
            .keys()
            .map(|name| name.as_str())
            .filter(|name| {
                name.starts_with("sec-ch-")
                    && !matches!(
                        *name,
                        "sec-ch-ua" | "sec-ch-ua-mobile" | "sec-ch-ua-platform"
                    )
            })
            .collect()
    }
}

type Requests = Arc<Mutex<Vec<Recorded>>>;

/// Starts a server that records every request and answers it with `respond`.
fn serve<F>(respond: F) -> (server::Server, Requests)
where
    F: Fn(&Recorded) -> http::Response<wreq::Body> + Clone + Send + 'static,
{
    let requests = Requests::default();
    let server = server::http({
        let requests = requests.clone();
        move |req| {
            let recorded = Recorded {
                method: req.method().clone(),
                path: req.uri().path().to_owned(),
                headers: req.headers().clone(),
            };
            let res = respond(&recorded);
            requests.lock().unwrap().push(recorded);
            async move { res }
        }
    });
    (server, requests)
}

fn response(status: StatusCode, headers: &[(&str, &str)]) -> http::Response<wreq::Body> {
    let mut res = http::Response::new(wreq::Body::default());
    *res.status_mut() = status;
    for (name, value) in headers {
        res.headers_mut().append(
            HeaderName::from_bytes(name.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }
    res
}

/// Refuses a request without `Sec-CH-UA-Arch`, naming the hints it wants.
fn challenge(req: &Recorded) -> http::Response<wreq::Body> {
    if req.header("sec-ch-ua-arch").is_some() {
        response(StatusCode::OK, &[])
    } else {
        response(
            StatusCode::FORBIDDEN,
            &[
                ("accept-ch", CHALLENGE_HINTS),
                ("critical-ch", CHALLENGE_HINTS),
            ],
        )
    }
}

fn chrome() -> Emulation {
    Emulation::builder()
        .profile(Profile::Chrome149)
        .platform(Platform::Windows)
        .build()
}

fn builder(emulation: Emulation) -> ClientBuilder {
    Client::builder()
        .emulation(emulation.clone())
        .layer(ClientHintsLayer::new(Hints::from_emulation(emulation)))
}

fn client(emulation: Emulation) -> Client {
    builder(emulation).build().expect("client")
}

#[tokio::test]
async fn test_restarts_for_critical_hints_and_remembers_them() {
    let (server, requests) = serve(challenge);
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    {
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].hints().is_empty());

        // Every hint the response accepted that Chrome supports, and no other.
        let restarted = &requests[1];
        assert_eq!(
            restarted.hints(),
            [
                "sec-ch-ua-full-version",
                "sec-ch-ua-arch",
                "sec-ch-ua-platform-version",
                "sec-ch-ua-model",
                "sec-ch-ua-bitness",
                "sec-ch-ua-full-version-list",
            ]
        );
        assert_eq!(restarted.header("sec-ch-ua-arch"), Some(r#""x86""#));
    }

    // The origin's hints are sent from the start now.
    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[2].hints(), requests[1].hints());
}

#[tokio::test]
async fn test_does_not_restart_without_accept_ch() {
    let (server, requests) =
        serve(|_| response(StatusCode::FORBIDDEN, &[("critical-ch", "Sec-CH-UA-Arch")]));
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    client.get(&url).send().await.expect("request");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| request.hints().is_empty()));
}

#[tokio::test]
async fn test_does_not_restart_for_a_critical_hint_that_is_not_accepted() {
    let (server, requests) = serve(|_| {
        response(
            StatusCode::FORBIDDEN,
            &[
                ("accept-ch", "Sec-CH-UA-Model"),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        )
    });
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    client.get(&url).send().await.expect("request");
    client.get(&url).send().await.expect("request");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].hints().is_empty());
    assert_eq!(requests[1].hints(), ["sec-ch-ua-model"]);
}

#[tokio::test]
async fn test_does_not_restart_when_the_critical_hints_were_sent() {
    let (server, requests) = serve(|req| match req.path.as_str() {
        "/accept" => response(StatusCode::OK, &[("accept-ch", "Sec-CH-UA-Arch")]),
        "/more" => response(
            StatusCode::OK,
            &[
                ("accept-ch", "Sec-CH-UA-Arch, Sec-CH-UA-Model"),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        ),
        _ => response(StatusCode::OK, &[]),
    });
    let client = client(chrome());
    let url = |path| format!("http://{}{path}", server.addr());

    for path in ["/accept", "/more", "/"] {
        client.get(url(path)).send().await.expect("request");
    }

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests[0].hints().is_empty());
    assert_eq!(requests[1].hints(), ["sec-ch-ua-arch"]);
    assert_eq!(requests[2].hints(), ["sec-ch-ua-arch", "sec-ch-ua-model"]);
}

#[tokio::test]
async fn test_accept_ch_replaces_and_clears_the_hints_of_an_origin() {
    let (server, requests) = serve(|req| match req.path.as_str() {
        "/all" => response(StatusCode::OK, &[("accept-ch", ALL_UA_HINTS)]),
        "/one" => response(StatusCode::OK, &[("accept-ch", "Sec-CH-UA-Bitness")]),
        "/none" => response(StatusCode::OK, &[("accept-ch", "")]),
        _ => response(StatusCode::OK, &[]),
    });
    let client = client(chrome());
    let url = |path| format!("http://{}{path}", server.addr());

    for path in ["/all", "/one", "/", "/none", "/"] {
        client.get(url(path)).send().await.expect("request");
    }

    let requests = requests.lock().unwrap();
    assert!(requests[0].hints().is_empty());
    assert_eq!(
        requests[1].hints(),
        [
            "sec-ch-ua-full-version",
            "sec-ch-ua-arch",
            "sec-ch-ua-platform-version",
            "sec-ch-ua-model",
            "sec-ch-ua-bitness",
            "sec-ch-ua-wow64",
            "sec-ch-ua-full-version-list",
            "sec-ch-ua-form-factors",
        ]
    );
    assert_eq!(requests[1].header("sec-ch-ua-wow64"), Some("?0"));
    assert_eq!(
        requests[1].header("sec-ch-ua-form-factors"),
        Some(r#""Desktop""#)
    );
    assert_eq!(requests[2].hints(), ["sec-ch-ua-bitness"]);
    // A response without `Accept-CH` keeps what the origin accepted.
    assert_eq!(requests[3].hints(), ["sec-ch-ua-bitness"]);
    assert!(requests[4].hints().is_empty());
}

#[tokio::test]
async fn test_keeps_hints_to_the_origin_that_asked() {
    let (asking, _) = serve(challenge);
    let (other, requests) = serve(|_| response(StatusCode::OK, &[]));
    let client = client(chrome());

    let res = client
        .get(format!("http://{}/", asking.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    client
        .get(format!("http://{}/", other.addr()))
        .send()
        .await
        .expect("request");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].hints().is_empty());
}

#[tokio::test]
async fn test_restarts_a_redirected_request_from_its_first_url() {
    let (server, requests) = serve(|req| match req.path.as_str() {
        "/start" => response(StatusCode::FOUND, &[("location", "/challenge")]),
        _ => challenge(req),
    });
    let client = builder(chrome())
        .redirect(redirect::Policy::default())
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/start", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.uri().path(), "/challenge");

    let requests = requests.lock().unwrap();
    let paths: Vec<_> = requests.iter().map(|req| req.path.as_str()).collect();
    assert_eq!(paths, ["/start", "/challenge", "/start", "/challenge"]);
    assert!(requests[0].hints().is_empty());
    assert!(requests[1].hints().is_empty());
    assert!(!requests[2].hints().is_empty());
    assert_eq!(requests[3].hints(), requests[2].hints());
}

#[tokio::test]
async fn test_restarts_for_critical_hints_of_a_redirect_response() {
    let (server, requests) = serve(|req| match req.path.as_str() {
        "/start" if req.header("sec-ch-ua-arch").is_none() => response(
            StatusCode::FOUND,
            &[
                ("location", "/next"),
                ("accept-ch", "Sec-CH-UA-Arch"),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        ),
        _ => response(StatusCode::OK, &[]),
    });
    let client = builder(chrome())
        .redirect(redirect::Policy::default())
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/start", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.uri().path(), "/start");

    let requests = requests.lock().unwrap();
    let paths: Vec<_> = requests.iter().map(|req| req.path.as_str()).collect();
    assert_eq!(paths, ["/start", "/next", "/start"]);
    assert_eq!(requests[2].hints(), ["sec-ch-ua-arch"]);
}

#[tokio::test]
async fn test_restarts_with_the_hints_of_every_response_in_a_chain() {
    let (server, requests) = serve(|req| match req.path.as_str() {
        "/start" if req.header("sec-ch-ua-arch").is_none() => response(
            StatusCode::FOUND,
            &[
                ("location", "/next"),
                ("accept-ch", "Sec-CH-UA-Arch"),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        ),
        "/start" => response(StatusCode::FOUND, &[("location", "/next")]),
        _ if req.header("sec-ch-ua-model").is_none() => response(
            StatusCode::FORBIDDEN,
            &[
                ("accept-ch", "Sec-CH-UA-Model"),
                ("critical-ch", "Sec-CH-UA-Model"),
            ],
        ),
        _ => response(StatusCode::OK, &[]),
    });
    let client = builder(chrome())
        .redirect(redirect::Policy::default())
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/start", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.uri().path(), "/next");

    let requests = requests.lock().unwrap();
    let paths: Vec<_> = requests.iter().map(|req| req.path.as_str()).collect();
    assert_eq!(paths, ["/start", "/next", "/start", "/next"]);
    assert_eq!(requests[2].hints(), ["sec-ch-ua-arch", "sec-ch-ua-model"]);
    assert_eq!(requests[3].hints(), ["sec-ch-ua-arch", "sec-ch-ua-model"]);
}

#[tokio::test]
async fn test_records_hints_for_the_origin_of_a_redirect_that_is_not_followed() {
    let (target, target_requests) = serve(|req| match req.path.as_str() {
        "/" => response(
            StatusCode::FOUND,
            &[("location", "/next"), ("accept-ch", "Sec-CH-UA-Arch")],
        ),
        _ => response(StatusCode::OK, &[]),
    });
    let location = format!("http://{}/", target.addr());
    let (start, start_requests) = serve({
        let location = location.clone();
        move |_| response(StatusCode::FOUND, &[("location", &location)])
    });
    let client = builder(chrome())
        // Follow the first redirect and return the second one.
        .redirect(redirect::Policy::custom(|attempt| {
            if attempt.previous.len() > 1 {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .expect("client");
    let start_url = format!("http://{}/", start.addr());

    let res = client.get(&start_url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.uri().to_string(), location);

    // Accept-CH of the returned 302 belongs to the target origin.
    client
        .get(&start_url)
        .redirect(redirect::Policy::none())
        .send()
        .await
        .expect("request");
    client
        .get(format!("{location}next"))
        .send()
        .await
        .expect("request");

    let start_requests = start_requests.lock().unwrap();
    assert_eq!(start_requests.len(), 2);
    assert!(start_requests[1].hints().is_empty());

    let target_requests = target_requests.lock().unwrap();
    assert_eq!(target_requests.len(), 2);
    assert_eq!(target_requests[1].hints(), ["sec-ch-ua-arch"]);
}

#[tokio::test]
async fn test_records_hints_for_the_origin_a_redirect_leads_to() {
    let (target, target_requests) = serve(challenge);
    let location = format!("http://{}/", target.addr());
    let (start, start_requests) = serve({
        let location = location.clone();
        move |_| response(StatusCode::FOUND, &[("location", &location)])
    });
    let client = builder(chrome())
        .redirect(redirect::Policy::default())
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/", start.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    // A request that goes to the first origin only doesn't carry the other origin's hints.
    let res = client
        .get(format!("http://{}/", start.addr()))
        .redirect(redirect::Policy::none())
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::FOUND);
    client.get(&location).send().await.expect("request");

    let target_requests = target_requests.lock().unwrap();
    assert_eq!(target_requests.len(), 3);
    assert!(target_requests[0].hints().is_empty());
    assert!(!target_requests[1].hints().is_empty());
    assert_eq!(target_requests[2].hints(), target_requests[1].hints());

    // The restart replays the first hop with the target's hints.
    let start_requests = start_requests.lock().unwrap();
    assert_eq!(start_requests.len(), 3);
    assert!(start_requests[0].hints().is_empty());
    assert_eq!(start_requests[1].hints(), target_requests[1].hints());
    assert!(start_requests[2].hints().is_empty());
}

#[tokio::test]
async fn test_does_not_resend_a_post() {
    let (server, requests) = serve(challenge);
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    let res = client.post(&url).body("a=b").send().await.expect("request");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // The hints the response asked for are still remembered.
    let res = client.post(&url).body("a=b").send().await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.method == Method::POST));
    assert!(requests[0].hints().is_empty());
    assert!(!requests[1].hints().is_empty());
}

#[tokio::test]
async fn test_restarts_a_head() {
    let (server, requests) = serve(challenge);
    let client = client(chrome());

    let res = client
        .head(format!("http://{}/", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|req| req.method == Method::HEAD));
}

#[tokio::test]
async fn test_does_not_resend_a_streaming_body() {
    let (server, requests) = serve(challenge);
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    let res = client
        .get(&url)
        .body(wreq::Body::wrap(wreq::Body::from("a=b")))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // The hints the response asked for are still remembered.
    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].hints().is_empty());
    assert!(!requests[1].hints().is_empty());
}

#[tokio::test]
async fn test_restarts_only_once() {
    // Every response asks for a critical hint the request before it didn't have.
    let (server, requests) = serve(|req| {
        if req.header("sec-ch-ua-arch").is_none() {
            response(
                StatusCode::FORBIDDEN,
                &[
                    ("accept-ch", "Sec-CH-UA-Arch"),
                    ("critical-ch", "Sec-CH-UA-Arch"),
                ],
            )
        } else if req.header("sec-ch-ua-model").is_none() {
            response(
                StatusCode::FORBIDDEN,
                &[
                    ("accept-ch", "Sec-CH-UA-Arch, Sec-CH-UA-Model"),
                    ("critical-ch", "Sec-CH-UA-Model"),
                ],
            )
        } else {
            response(StatusCode::OK, &[])
        }
    });
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(requests.lock().unwrap().len(), 2);

    // What the second response asked for is remembered all the same.
    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1].hints(), ["sec-ch-ua-arch"]);
    assert_eq!(requests[2].hints(), ["sec-ch-ua-arch", "sec-ch-ua-model"]);
}

#[tokio::test]
async fn test_restarts_without_remembering_when_no_origins_are_kept() {
    let (server, requests) = serve(challenge);
    let emulation = chrome();
    let client = Client::builder()
        .emulation(emulation.clone())
        .layer(ClientHintsLayer::new(Hints::from_emulation(emulation)).max_origins(0))
        .build()
        .expect("client");
    let url = format!("http://{}/", server.addr());

    for _ in 0..2 {
        let res = client.get(&url).send().await.expect("request");
        assert_eq!(res.status(), StatusCode::OK);
    }

    let requests = requests.lock().unwrap();
    let hinted: Vec<_> = requests.iter().map(|req| !req.hints().is_empty()).collect();
    assert_eq!(hinted, [false, true, false, true]);
}

#[tokio::test]
async fn test_sends_cookies_of_the_refused_response() {
    let (server, requests) = serve(|req| {
        let mut res = challenge(req);
        if res.status() == StatusCode::FORBIDDEN {
            res.headers_mut().insert(
                "set-cookie",
                HeaderValue::from_static("challenge=1; Path=/"),
            );
        }
        res
    });
    let client = builder(chrome())
        .cookie_store(true)
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].header("cookie"), None);
    assert_eq!(requests[1].header("cookie"), Some("challenge=1"));
}

#[tokio::test]
async fn test_sends_nothing_for_browsers_without_client_hints() {
    let (server, requests) = serve(challenge);
    let client = client(Emulation::builder().profile(Profile::Firefox152).build());
    let url = format!("http://{}/", server.addr());

    for _ in 0..2 {
        let res = client.get(&url).send().await.expect("request");
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| request.hints().is_empty()));
}

#[tokio::test]
async fn test_derives_hints_for_a_per_request_emulation() {
    let (server, requests) = serve(challenge);
    let client = Client::builder()
        .layer(ClientHintsLayer::new(None))
        .build()
        .expect("client");
    let url = format!("http://{}/", server.addr());

    // The client has no identity of its own.
    let res = client.get(&url).send().await.expect("request");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let mac = Emulation::builder()
        .profile(Profile::Chrome154)
        .platform(Platform::MacOS)
        .build();
    let res = client
        .get(&url)
        .emulation(mac)
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].header("sec-ch-ua"), None);
    assert!(requests[1].hints().is_empty());
    assert_eq!(requests[2].header("sec-ch-ua-arch"), Some(r#""arm""#));
    assert_eq!(
        requests[2].header("sec-ch-ua-full-version"),
        Some(r#""154.0.8037.100""#)
    );
}

#[tokio::test]
async fn test_prefers_the_identity_of_the_request() {
    let (server, requests) = serve(|_| response(StatusCode::OK, &[("accept-ch", ALL_UA_HINTS)]));
    let client = client(chrome());
    let url = format!("http://{}/", server.addr());

    client.get(&url).send().await.expect("request");

    // Another browser on the same client gets its own hints, or none if it has none.
    let linux = Emulation::builder()
        .profile(Profile::Chrome143)
        .platform(Platform::Linux)
        .build();
    client
        .get(&url)
        .emulation(linux)
        .send()
        .await
        .expect("request");
    client
        .get(&url)
        .header(USER_AGENT, "curl/8.0")
        .send()
        .await
        .expect("request");
    // A hint the request sets is kept.
    client
        .get(&url)
        .header("sec-ch-ua-arch", r#""arm""#)
        .send()
        .await
        .expect("request");

    let requests = requests.lock().unwrap();
    assert!(requests[0].hints().is_empty());
    assert_eq!(
        requests[1].header("sec-ch-ua-platform-version"),
        Some(r#""6.8.0""#)
    );
    assert_eq!(
        requests[1].header("sec-ch-ua-full-version"),
        Some(r#""143.0.7499.192""#)
    );
    assert!(requests[2].hints().is_empty());
    assert_eq!(requests[3].header("sec-ch-ua-arch"), Some(r#""arm""#));
    assert_eq!(requests[3].header("sec-ch-ua-bitness"), Some(r#""64""#));
}

#[tokio::test]
async fn test_does_not_restart_when_the_request_sets_the_critical_hint() {
    let (server, requests) = serve(|_| {
        response(
            StatusCode::OK,
            &[
                ("accept-ch", "Sec-CH-UA-Arch"),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        )
    });
    let client = client(chrome());

    client
        .get(format!("http://{}/", server.addr()))
        .header("sec-ch-ua-arch", r#""arm""#)
        .send()
        .await
        .expect("request");

    assert_eq!(requests.lock().unwrap().len(), 1);
}

// The order Chrome 154 sent these headers in, first with hints its origin had accepted and then
// on a request restarted for `Critical-CH`.
const HINTED_ORDER: &[&str] = &[
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

const RESTARTED_ORDER: &[&str] = &[
    "upgrade-insecure-requests",
    "user-agent",
    "accept",
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
    "sec-fetch-site",
    "sec-fetch-mode",
    "sec-fetch-user",
    "sec-fetch-dest",
    #[cfg(feature = "emulation-compression")]
    "accept-encoding",
    "accept-language",
    "priority",
];

/// Refuses a request without `Sec-CH-UA-Arch`, accepting every UA hint.
fn challenge_all(req: &Recorded) -> http::Response<wreq::Body> {
    if req.header("sec-ch-ua-arch").is_some() {
        response(StatusCode::OK, &[])
    } else {
        response(
            StatusCode::FORBIDDEN,
            &[
                ("accept-ch", ALL_UA_HINTS),
                ("critical-ch", "Sec-CH-UA-Arch"),
            ],
        )
    }
}

/// The header names of a request in the order they arrived, without `host`.
fn names(req: &Recorded) -> Vec<&str> {
    req.headers
        .keys()
        .map(|name| name.as_str())
        .filter(|name| *name != "host")
        .collect()
}

#[tokio::test]
async fn test_writes_hints_in_chrome_order() {
    for version in [Version::HTTP_11, Version::HTTP_2] {
        let (server, requests) = serve(challenge_all);
        let client = client(chrome());
        let url = format!("http://{}/", server.addr());

        for _ in 0..2 {
            let res = client
                .get(&url)
                .version(version)
                .send()
                .await
                .expect("request");
            assert_eq!(res.status(), StatusCode::OK);
            assert_eq!(res.version(), version);
        }

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(names(&requests[1]), RESTARTED_ORDER);
        assert_eq!(names(&requests[2]), HINTED_ORDER);
    }
}

#[tokio::test]
async fn test_orig_headers_of_the_client_decide_the_order() {
    use wreq::header::OrigHeaderMap;

    let (server, requests) = serve(challenge_all);
    let mut order = OrigHeaderMap::new();
    for name in [
        "user-agent",
        "sec-ch-ua-model",
        "sec-ch-ua-arch",
        "sec-ch-ua",
    ] {
        order.insert(name);
    }
    let client = builder(chrome())
        .orig_headers(order)
        .build()
        .expect("client");

    let res = client
        .get(format!("http://{}/", server.addr()))
        .send()
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);

    let requests = requests.lock().unwrap();
    assert_eq!(
        names(&requests[1])[..4],
        [
            "user-agent",
            "sec-ch-ua-model",
            "sec-ch-ua-arch",
            "sec-ch-ua"
        ]
    );
}
