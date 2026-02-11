use wreq::Client;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Example 1: Basic emulation - Safari26
    println!("=== Example 1: Basic Safari26 Emulation ===");
    let client1 = Client::builder()
        .emulation(Emulation::Safari26)
        .cert_verification(false)
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
    let emulation2 = EmulationOption::builder()
        .emulation(Emulation::Firefox128)
        .emulation_os(EmulationOS::Windows)
        .skip_http2(true)
        .build();

    let client2 = Client::builder()
        .emulation(emulation2)
        .http1_only()
        .cert_verification(false)
        .build()?;

    let text2 = client2
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}\n", text2);

    // Example 3: Random device emulation
    println!("=== Example 3: Random Device Emulation ===");
    let client3 = Client::builder()
        .emulation(Emulation::random())
        .cert_verification(false)
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
