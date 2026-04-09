use wreq::Client;
use wreq_util::{Emulation, Platform, Profile};

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Example 1: Basic emulation - Safari26
    println!("=== Example 1: Basic Safari26 Emulation ===");
    let client = Client::builder()
        .emulation(Emulation::Chrome145)
        .tls_cert_verification(false)
        .build()?;

    let text = client
        .get("https://tls.browserleaks.com/")
        .send()
        .await?
        .text()
        .await?;

    println!("{}\n", text);

    // Example 2: Advanced emulation with options - OkHttp5
    println!("=== Example 2: Firefox128 with Custom Options ===");
    let emulation = Emulation::builder()
        .profile(Profile::OkHttp5)
        .platform(Platform::Windows)
        .http2(false)
        .build();

    let client = Client::builder()
        .emulation(emulation)
        .tls_cert_verification(false)
        .build()?;

    let text = client
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}\n", text);

    // Example 3: Random device emulation
    println!("=== Example 3: Random Profile Emulation ===");
    let client = Client::builder()
        .emulation(Emulation::random())
        .tls_cert_verification(false)
        .build()?;

    let text = client
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}", text);

    Ok(())
}
