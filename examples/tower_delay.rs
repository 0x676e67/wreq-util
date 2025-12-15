use std::time::{Duration, Instant};

use wreq::Client;
use wreq_util::tower::delay::DelayLayer;

#[tokio::main]
async fn main() -> Result<(), wreq::Error> {
    // Build a client with a 1 second delay between requests
    let client = Client::builder()
        .layer(DelayLayer::new(Duration::from_secs(1)))
        .cert_verification(false)
        .build()?;

    let url = "https://httpbin.org/get";
    println!("Sending 3 requests to {} using DelayLayer(1s)", url);

    let mut prev_start = Instant::now();

    for i in 1..=3 {
        let start = Instant::now();
        let delta = start.duration_since(prev_start);
        println!("Request #{}, start (+{:.3}s)", i, delta.as_secs_f64());

        let _ = client.get(url).send().await?;

        println!(
            "Request #{}, completed in {:.3}s",
            i,
            start.elapsed().as_secs_f64()
        );

        prev_start = start;
    }

    Ok(())
}
