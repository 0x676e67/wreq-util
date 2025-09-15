use wreq::Client;
use wreq_util::Emulation;

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Build a client to emulation Firefox136
    let client = Client::builder()
        .emulation(Emulation::Firefox109)
        .cert_verification(false)
        .build()?;

    // Use the API you're already familiar with
    let text = client
        .get("https://tls.peet.ws/api/all")
        .send()
        .await?
        .text()
        .await?;

    println!("{}", text);

    Ok(())
}
