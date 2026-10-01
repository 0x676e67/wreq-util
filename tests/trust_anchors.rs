#![cfg(all(feature = "emulation", not(target_arch = "wasm32")))]

use std::time::Duration;

use tokio::{io::AsyncReadExt, net::TcpListener, time::timeout};
use wreq::{Client, IntoEmulation};
use wreq_util::Profile;

#[tokio::test]
async fn reused_native_emulation_retains_trust_anchors_on_new_connections() {
    let request_client = Client::builder().no_proxy().build().unwrap();
    for profile in [Profile::Chrome152, Profile::Chrome153, Profile::Chrome154] {
        let native = profile.into_emulation();
        let expected = native
            .tls_options
            .as_ref()
            .unwrap()
            .trust_anchors
            .as_ref()
            .unwrap();
        let client = Client::builder()
            .no_proxy()
            .emulation(native.clone())
            .build()
            .unwrap();
        for _ in 0..2 {
            assert_eq!(
                capture_trust_anchors(&client, None).await,
                expected.as_ref()
            );
            assert_eq!(
                capture_trust_anchors(&request_client, Some(&native)).await,
                expected.as_ref()
            );
        }
    }
}

// Each listener forces a new connection and closes after ClientHello. No
// certificate or external service is needed to inspect the extension bytes.
async fn capture_trust_anchors(client: &Client, native: Option<&wreq::Emulation>) -> Vec<u8> {
    timeout(Duration::from_secs(5), async {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let mut request = client.get(format!("https://{}/", listener.local_addr().unwrap()));
        if let Some(native) = native {
            request = request.emulation(native.clone());
        }
        let capture = async {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut handshake = Vec::new();
            loop {
                let mut header = [0; 5];
                socket.read_exact(&mut header).await.unwrap();
                assert_eq!(header[0], 22, "expected a TLS handshake record");
                let length = usize::from(u16::from_be_bytes([header[3], header[4]]));
                let offset = handshake.len();
                handshake.resize(offset + length, 0);
                socket.read_exact(&mut handshake[offset..]).await.unwrap();
                if handshake.len() >= 4 {
                    assert_eq!(handshake[0], 1, "expected ClientHello");
                    let length = handshake[1..4]
                        .iter()
                        .fold(0usize, |n, &byte| (n << 8) | usize::from(byte));
                    if handshake.len() >= length + 4 {
                        return trust_anchors_from_client_hello(&handshake[4..length + 4]);
                    }
                }
            }
        };
        let (result, ids) = tokio::join!(request.send(), capture);
        assert!(result.unwrap_err().is_connect());
        ids
    })
    .await
    .expect("ClientHello capture timed out")
}

fn trust_anchors_from_client_hello(mut hello: &[u8]) -> Vec<u8> {
    take(&mut hello, 2 + 32); // Legacy version and random.
    take_vector(&mut hello, 1); // Session ID.
    take_vector(&mut hello, 2); // Cipher suites.
    take_vector(&mut hello, 1); // Compression methods.
    let mut extensions = take_vector(&mut hello, 2);
    while !extensions.is_empty() {
        let kind = take(&mut extensions, 2);
        let mut data = take_vector(&mut extensions, 2);
        if u16::from_be_bytes([kind[0], kind[1]]) == 51764 {
            let ids = take_vector(&mut data, 2);
            assert!(data.is_empty());
            return ids.to_vec();
        }
    }
    panic!("missing trust_anchors extension");
}

fn take<'a>(input: &mut &'a [u8], length: usize) -> &'a [u8] {
    let (value, rest) = input.split_at(length);
    *input = rest;
    value
}

fn take_vector<'a>(input: &mut &'a [u8], prefix: usize) -> &'a [u8] {
    let length = take(input, prefix)
        .iter()
        .fold(0usize, |n, &byte| (n << 8) | usize::from(byte));
    take(input, length)
}
