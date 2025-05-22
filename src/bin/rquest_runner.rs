use clap::{Parser, command};
use rquest::{Client, header, Proxy};
use rquest_util::Emulation;
use serde::Serialize;
use std::str::FromStr;
use url::Url;

/// A command-line HTTP client with browser emulation capabilities
#[derive(Parser, Debug)]
#[command(
    name = "rquest_runner",
    author = "rquest-util",
    version,
    about = "Make HTTP requests with browser emulation",
    long_about = "A command-line tool for making HTTP requests with browser fingerprint emulation, proxy support, and cookie handling.",
    after_help = "Examples:\n\
    - Basic request:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com\n\
    \n\
    - With proxy:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com -x 127.0.0.1:8080:user:pass\n\
    \n\
    - With cookies:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com -c \"session=123\" -C \"user=john; theme=dark\"\n"
)]
struct Args {
    /// Browser profile to emulate (e.g., Chrome136, Firefox136, Edge134)
    #[arg(short = 'P', long, value_name = "PROFILE")]
    profile: String,

    /// HTTP method (GET, POST, PUT, DELETE, HEAD)
    #[arg(short, long, value_name = "METHOD")]
    method: String,

    /// Target URL to send the request to
    #[arg(short, long, value_name = "URL")]
    url: String,

    /// Proxy configuration in format ip:port:username:password
    #[arg(
        short = 'x',
        long,
        value_name = "PROXY",
        help = "Proxy in format ip:port:username:password (e.g., 127.0.0.1:8080:user:pass)"
    )]
    proxy: Option<String>,

    /// Single cookie in format name=value
    #[arg(
        short = 'c',
        long,
        value_name = "COOKIE",
        help = "Single cookie in format name=value (e.g., \"session=abc123\")"
    )]
    cookie: Option<String>,

    /// Multiple cookies in format "name1=value1; name2=value2"
    #[arg(
        short = 'C',
        long,
        value_name = "COOKIES",
        help = "Multiple cookies in format \"name1=value1; name2=value2\""
    )]
    cookies: Option<String>,
}

#[derive(Serialize)]
struct Response {
    response_code: u16,
    headers: Vec<(String, String)>,
    body: String,
}

struct ProxyConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl FromStr for ProxyConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err("Proxy must be in format ip:port:username:password".to_string());
        }

        let port = parts[1].parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?;

        Ok(ProxyConfig {
            host: parts[0].to_string(),
            port,
            username: parts[2].to_string(),
            password: parts[3].to_string(),
        })
    }
}

fn parse_cookie(cookie_str: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = cookie_str.split('=').collect();
    if parts.len() != 2 {
        return Err("Cookie must be in format name=value".to_string());
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

fn parse_cookies(cookies_str: &str) -> Result<Vec<(String, String)>, String> {
    cookies_str
        .split(';')
        .map(parse_cookie)
        .collect()
}

fn parse_emulation(profile: &str) -> Result<Emulation, String> {
    match profile {
        // Chrome versions
        "Chrome136" => Ok(Emulation::Chrome136),
        "Chrome135" => Ok(Emulation::Chrome135),
        "Chrome134" => Ok(Emulation::Chrome134),
        "Chrome133" => Ok(Emulation::Chrome133),
        "Chrome132" => Ok(Emulation::Chrome132),
        "Chrome131" => Ok(Emulation::Chrome131),
        "Chrome130" => Ok(Emulation::Chrome130),
        "Chrome129" => Ok(Emulation::Chrome129),
        "Chrome128" => Ok(Emulation::Chrome128),
        "Chrome127" => Ok(Emulation::Chrome127),
        "Chrome126" => Ok(Emulation::Chrome126),
        "Chrome124" => Ok(Emulation::Chrome124),
        "Chrome123" => Ok(Emulation::Chrome123),
        "Chrome120" => Ok(Emulation::Chrome120),
        "Chrome119" => Ok(Emulation::Chrome119),
        "Chrome118" => Ok(Emulation::Chrome118),
        "Chrome117" => Ok(Emulation::Chrome117),
        "Chrome116" => Ok(Emulation::Chrome116),
        "Chrome114" => Ok(Emulation::Chrome114),
        "Chrome110" => Ok(Emulation::Chrome110),
        "Chrome109" => Ok(Emulation::Chrome109),
        "Chrome108" => Ok(Emulation::Chrome108),
        "Chrome107" => Ok(Emulation::Chrome107),
        "Chrome106" => Ok(Emulation::Chrome106),
        "Chrome105" => Ok(Emulation::Chrome105),
        "Chrome104" => Ok(Emulation::Chrome104),
        "Chrome101" => Ok(Emulation::Chrome101),
        "Chrome100" => Ok(Emulation::Chrome100),

        // Firefox versions
        "Firefox136" => Ok(Emulation::Firefox136),
        "FirefoxPrivate136" => Ok(Emulation::FirefoxPrivate136),
        "Firefox135" => Ok(Emulation::Firefox135),
        "FirefoxPrivate135" => Ok(Emulation::FirefoxPrivate135),
        "FirefoxAndroid135" => Ok(Emulation::FirefoxAndroid135),
        "Firefox133" => Ok(Emulation::Firefox133),
        "Firefox128" => Ok(Emulation::Firefox128),
        "Firefox117" => Ok(Emulation::Firefox117),
        "Firefox109" => Ok(Emulation::Firefox109),

        // Edge versions
        "Edge134" => Ok(Emulation::Edge134),
        "Edge131" => Ok(Emulation::Edge131),
        "Edge127" => Ok(Emulation::Edge127),
        "Edge122" => Ok(Emulation::Edge122),
        "Edge101" => Ok(Emulation::Edge101),

        // Safari versions
        "Safari18_3_1" => Ok(Emulation::Safari18_3_1),
        "Safari18_3" => Ok(Emulation::Safari18_3),
        "Safari18_2" => Ok(Emulation::Safari18_2),
        "Safari18" => Ok(Emulation::Safari18),
        "SafariIPad18" => Ok(Emulation::SafariIPad18),
        "Safari17_5" => Ok(Emulation::Safari17_5),
        "Safari17_4_1" => Ok(Emulation::Safari17_4_1),
        "Safari17_2_1" => Ok(Emulation::Safari17_2_1),
        "Safari17_0" => Ok(Emulation::Safari17_0),
        "Safari16_5" => Ok(Emulation::Safari16_5),
        "Safari16" => Ok(Emulation::Safari16),
        "Safari15_6_1" => Ok(Emulation::Safari15_6_1),
        "Safari15_5" => Ok(Emulation::Safari15_5),
        "Safari15_3" => Ok(Emulation::Safari15_3),
        "SafariIos16_5" => Ok(Emulation::SafariIos16_5),
        "SafariIos17_4_1" => Ok(Emulation::SafariIos17_4_1),
        "SafariIos17_2" => Ok(Emulation::SafariIos17_2),

        // OkHttp versions
        "OkHttp5" => Ok(Emulation::OkHttp5),
        "OkHttp4_12" => Ok(Emulation::OkHttp4_12),
        "OkHttp4_10" => Ok(Emulation::OkHttp4_10),
        "OkHttp4_9" => Ok(Emulation::OkHttp4_9),
        "OkHttp3_14" => Ok(Emulation::OkHttp3_14),
        "OkHttp3_13" => Ok(Emulation::OkHttp3_13),
        "OkHttp3_11" => Ok(Emulation::OkHttp3_11),
        "OkHttp3_9" => Ok(Emulation::OkHttp3_9),

        _ => {
            let available_profiles = [
                // Chrome
                "Chrome100-Chrome136",
                // Firefox
                "Firefox109, Firefox117, Firefox128, Firefox133-136",
                "FirefoxPrivate135-136, FirefoxAndroid135",
                // Edge
                "Edge101, Edge122, Edge127, Edge131, Edge134",
                // Safari
                "Safari15.3-18.3.1, Safari16-18",
                "SafariIPad18",
                "SafariIos16.5, SafariIos17.2, SafariIos17.4.1",
                // OkHttp
                "OkHttp3.9-5.0"
            ].join("\n  ");
            
            Err(format!(
                "Invalid profile: {}\n\nAvailable profiles:\n  {}", 
                profile, 
                available_profiles
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse the emulation profile
    let emulation = parse_emulation(&args.profile)?;

    // Start building the client
    let mut builder = Client::builder().emulation(emulation);

    // Configure proxy if provided
    if let Some(proxy_str) = args.proxy {
        let proxy = ProxyConfig::from_str(&proxy_str)?;
        
        // Create proxy configuration with authentication
        let proxy_url = format!("http://{}:{}@{}:{}", 
            proxy.username,
            proxy.password,
            proxy.host,
            proxy.port
        );
        let proxy_config = Proxy::http(proxy_url)?;
        
        builder = builder.proxy(proxy_config);
    }

    // Build the client
    let client = builder.build()?;

    // Create the request based on method
    let mut request = match args.method.to_uppercase().as_str() {
        "GET" => client.get(&args.url),
        "POST" => client.post(&args.url),
        "PUT" => client.put(&args.url),
        "DELETE" => client.delete(&args.url),
        "HEAD" => client.head(&args.url),
        _ => return Err(format!("Unsupported method: {}", args.method).into()),
    };

    // Add cookies if provided
    let mut cookie_header = String::new();

    // Handle single cookie
    if let Some(cookie_str) = args.cookie {
        let (name, value) = parse_cookie(&cookie_str)?;
        cookie_header.push_str(&format!("{}={}", name, value));
    }

    // Handle multiple cookies
    if let Some(cookies_str) = args.cookies {
        let cookies = parse_cookies(&cookies_str)?;
        for (i, (name, value)) in cookies.iter().enumerate() {
            if i > 0 || !cookie_header.is_empty() {
                cookie_header.push_str("; ");
            }
            cookie_header.push_str(&format!("{}={}", name, value));
        }
    }

    // Add cookie header if we have any cookies
    if !cookie_header.is_empty() {
        request = request.header(header::COOKIE, cookie_header);
    }

    // Send the request
    let resp = request.send().await?;
    
    // Extract status code
    let status = resp.status().as_u16();
    
    // Extract headers
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    // Get the body
    let body = resp.text().await?;

    // Create and serialize the response
    let response = Response {
        response_code: status,
        headers,
        body,
    };

    // Print as JSON
    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
} 