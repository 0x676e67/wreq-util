use wreq::Client;
use wreq_util::{
    Emulation, Platform, Profile,
    tower::client_hints::{ClientHintsLayer, Hints},
};

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Pass a URL that answers with `Accept-CH`, or with `Critical-CH` to see a restart
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://www.google.com/".to_owned());

    let emulation = Emulation::builder()
        .profile(Profile::Chrome149)
        .platform(Platform::Windows)
        .build();

    // Derive the hints from the emulation's default headers
    let hints = Hints::from_emulation(emulation.clone());
    if let Some(hints) = &hints {
        println!("Hints for origins that ask for them:");
        for (name, value) in hints.iter() {
            println!("  {name}: {value:?}");
        }
    }

    // `Critical-CH` responses often set a cookie, so keep cookies across the restart
    let client = Client::builder()
        .emulation(emulation)
        .layer(ClientHintsLayer::new(hints))
        .cookie_store(true)
        .build()?;

    // The first request is restarted if the response has `Critical-CH`,
    // the second one carries the hints the origin accepted from the start
    for i in 1..=2 {
        let resp = client.get(&url).send().await?;
        println!(
            "\nRequest #{}: {} {}\n  accept-ch: {:?}\n  critical-ch: {:?}",
            i,
            resp.status(),
            resp.uri(),
            resp.headers().get("accept-ch"),
            resp.headers().get("critical-ch"),
        );
    }

    Ok(())
}
