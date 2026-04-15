use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

use http::Uri;
use wreq::cookie::CookieStore;
use wreq_util::cookie::Jar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jar = Jar::default();
    let login_uri: Uri = "https://example.com/account/login".parse()?;

    jar.add(
        "sid=abc123; Domain=example.com; Path=/account; Secure; HttpOnly",
        &login_uri,
    );
    jar.add("theme=dark; Domain=example.com; Path=/", &login_uri);

    let json = jar.to_json()?;
    println!("Serialized JSON:\n{json}\n");

    let path = temp_file_path();
    jar.save_to_file(&path)?;

    let file_content = fs::read_to_string(&path)?;
    println!("Saved file: {}", path.display());
    println!("File content:\n{file_content}\n");

    let restored = Jar::default();
    let loaded = restored.load_from_file(&path)?;
    println!("Loaded {loaded} cookies from file");

    let request_uri: Uri = "https://example.com/account/profile".parse()?;
    let cookie_header = restored.cookies(&request_uri, http::Version::HTTP_11);
    println!("Cookies for {request_uri}: {cookie_header:?}");

    let parsed: Vec<String> = serde_json::from_str(&json)?;
    println!("JSON layout is a string array of Set-Cookie values:");
    for (index, cookie) in parsed.iter().enumerate() {
        println!("  [{}] {}", index, cookie);
    }

    fs::remove_file(path)?;

    Ok(())
}

fn temp_file_path() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("wreq-util-cookie-example-{suffix}.json"))
}
