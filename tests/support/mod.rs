#![allow(dead_code)]
pub mod delay_layer;
pub mod delay_server;
pub mod server;

use rquest::Client;
use std::{sync::LazyLock, time::Duration};

// TODO: remove once done converting to new support server?
#[allow(unused)]
pub static DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .unwrap()
});

#[allow(unused_macros)]
macro_rules! test_emulation {
    ($test_name:ident, $emulation:expr, $ja4:expr, $akamai_hash:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let client = CLIENT.cloned();
            client.update().emulation($emulation).apply().unwrap();

            let resp = client
                .get("https://tls.browserleaks.com/")
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), rquest::StatusCode::OK);
            let content = resp.text().await.unwrap();

            let conditional = $ja4.iter().any(|&ja4| content.contains(ja4));
            if !conditional {
                println!("{}", content);
            }
            assert!(conditional);

            let conditional = content.contains($akamai_hash);
            if !conditional {
                println!("{}", content);
            }
            assert!(conditional);

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    };
}
