use wreq::Client;
use wreq_util::{Emulation, Platform, Profile};

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Example 1: Basic emulation - Safari26
    println!("=== Example 1: Basic Safari26 Emulation ===");
    let client1 = Client::builder()
        .emulation(Emulation::Chrome145)
        .tls_cert_verification(false)
        .build()?;

    let text1 = client1
        .get("https://tls.browserleaks.com/")
        .send()
        .await?
        .text()
        .await?;

    println!("{}\n", text1);

    // Example 2: Advanced emulation with options - Firefox128
    println!("=== Example 2: Firefox128 with Custom Options ===");
    let emulation = Emulation::builder()
        .profile(Profile::Firefox128)
        .platform(Platform::Windows)
        .http2(true)
        .build();

    let client2 = Client::builder()
        .emulation(emulation)
        .http1_only()
        .tls_cert_verification(false)
        .build()?;

    let text2 = client2
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}\n", text2);

    // Example 3: Random device emulation
    println!("=== Example 3: Random Profile Emulation ===");
    let client3 = Client::builder()
        .emulation(Emulation::random())
        .tls_cert_verification(false)
        .build()?;

    let text3 = client3
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}", text3);

    Ok(())
}
