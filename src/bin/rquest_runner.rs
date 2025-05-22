use clap::{Parser, command, ValueEnum};
use rquest::{Client, header, Proxy};
use rquest_util::Emulation;
use serde::Serialize;
use std::str::FromStr;
use std::fs::{self, File};
use std::io::Write;
use chrono::Local;
use log::{info, debug};
use url;

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            _ => Err(format!("Invalid HTTP method: {}. Valid values are: GET, POST, PUT, DELETE, HEAD", s)),
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::HEAD => write!(f, "HEAD"),
        }
    }
}

#[derive(Debug, Clone)]
struct ValidUrl(String);

impl FromStr for ValidUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        url::Url::parse(s)
            .map(|_| ValidUrl(s.to_string()))
            .map_err(|e| format!("Invalid URL: {}", e))
    }
}

impl From<String> for ValidUrl {
    fn from(s: String) -> Self {
        ValidUrl(s)
    }
}

impl From<&str> for ValidUrl {
    fn from(s: &str) -> Self {
        ValidUrl(s.to_string())
    }
}

#[derive(Debug, Clone)]
struct ValidCookie(String);

impl FromStr for ValidCookie {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('=') {
            Ok(ValidCookie(s.to_string()))
        } else {
            Err("Cookie must be in format name=value".to_string())
        }
    }
}

impl From<String> for ValidCookie {
    fn from(s: String) -> Self {
        ValidCookie(s)
    }
}

impl From<&str> for ValidCookie {
    fn from(s: &str) -> Self {
        ValidCookie(s.to_string())
    }
}

#[derive(Debug, Clone)]
struct ValidCookies(String);

impl FromStr for ValidCookies {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for cookie in s.split(';') {
            if !cookie.trim().contains('=') {
                return Err(format!("Invalid cookie format in: {}", cookie));
            }
        }
        Ok(ValidCookies(s.to_string()))
    }
}

impl From<String> for ValidCookies {
    fn from(s: String) -> Self {
        ValidCookies(s)
    }
}

impl From<&str> for ValidCookies {
    fn from(s: &str) -> Self {
        ValidCookies(s.to_string())
    }
}

#[derive(Debug, Clone)]
struct ValidHeader(String);

impl FromStr for ValidHeader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            Ok(ValidHeader(s.to_string()))
        } else {
            Err("Header must be in format name:value".to_string())
        }
    }
}

impl From<String> for ValidHeader {
    fn from(s: String) -> Self {
        ValidHeader(s)
    }
}

impl From<&str> for ValidHeader {
    fn from(s: &str) -> Self {
        ValidHeader(s.to_string())
    }
}

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
      rquest_runner -P Chrome136 -m GET -u https://example.com -xhost 127.0.0.1 -xport 8080 -xuser user -xpass pass\n\
    \n\
    - With cookies:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com -c \"session=123\" -C \"user=john; theme=dark\"\n\
    \n\
    - With custom headers:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com -H \"Authorization: Bearer token\" -H \"X-Custom: value\"\n\
    \n\
    - With request body:\n\
      rquest_runner -P Chrome136 -m POST -u https://example.com -b '{\"key\": \"value\"}'\n\
    \n\
    - With verbose logging:\n\
      rquest_runner -P Chrome136 -m GET -u https://example.com --verbose\n"
)]
struct Args {
    /// Browser profile to emulate (e.g., Chrome136, Firefox136, Edge134)
    #[arg(short = 'P', long, value_name = "PROFILE")]
    profile: String,

    /// HTTP method (GET, POST, PUT, DELETE, HEAD)
    #[arg(short, long, value_name = "METHOD", value_parser = |s: &str| -> Result<HttpMethod, String> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            _ => Err(format!("Invalid HTTP method: {}. Valid values are: GET, POST, PUT, DELETE, HEAD", s))
        }
    })]
    method: HttpMethod,

    /// Target URL to send the request to
    #[arg(short, long, value_name = "URL")]
    url: ValidUrl,

    /// Request body (for POST, PUT, etc.)
    #[arg(
        short = 'b',
        long = "body",
        value_name = "BODY",
        help = "Request body (e.g., '{\"key\": \"value\"}' for JSON)"
    )]
    body: Option<String>,

    /// Proxy host address
    #[arg(
        long = "xhost",
        value_name = "HOST",
        help = "Proxy host address (e.g., 127.0.0.1)",
        requires = "proxy_port"
    )]
    proxy_host: Option<String>,

    /// Proxy port number
    #[arg(
        long = "xport",
        value_name = "PORT",
        help = "Proxy port number (e.g., 8080)",
        requires = "proxy_host"
    )]
    proxy_port: Option<u16>,

    /// Proxy username
    #[arg(
        long = "xuser",
        value_name = "USERNAME",
        help = "Proxy username",
        requires = "proxy_host"
    )]
    proxy_username: Option<String>,

    /// Proxy password
    #[arg(
        long = "xpass",
        value_name = "PASSWORD",
        help = "Proxy password",
        requires = "proxy_username"
    )]
    proxy_password: Option<String>,

    /// Single cookie in format name=value
    #[arg(
        short = 'c',
        long,
        value_name = "COOKIE",
        help = "Single cookie in format name=value (e.g., \"session=abc123\")"
    )]
    cookie: Option<ValidCookie>,

    /// Multiple cookies in format "name1=value1; name2=value2"
    #[arg(
        short = 'C',
        long,
        value_name = "COOKIES",
        help = "Multiple cookies in format \"name1=value1; name2=value2\""
    )]
    cookies: Option<ValidCookies>,

    /// Custom headers in format "name:value"
    #[arg(
        short = 'H',
        long = "header",
        value_name = "HEADER",
        help = "Custom header in format \"name:value\" (e.g., \"Authorization: Bearer token\")",
        number_of_values = 1
    )]
    headers: Vec<ValidHeader>,

    /// Enable verbose logging to a timestamped log file
    #[arg(long)]
    verbose: bool,
}

#[derive(Serialize)]
struct Response {
    response_code: u16,
    headers: Vec<(String, String)>,
    body: String,
}

fn parse_header(header_str: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = header_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Header must be in format name:value".to_string());
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
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

fn setup_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        // Create logs directory if it doesn't exist
        fs::create_dir_all("logs")?;

        // Create timestamped log file
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let log_file = format!("logs/rquest_{}.log", timestamp);
        
        // Initialize logger
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format(|buf, record| {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                writeln!(
                    buf,
                    "{} [{}] {}: {}",
                    timestamp,
                    record.level(),
                    record.target(),
                    record.args()
                )
            })
            .target(env_logger::Target::Pipe(Box::new(File::create(&log_file)?)))
            .init();

        info!("Verbose logging enabled. Log file: {}", log_file);
    } else {
        // Initialize minimal logger
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.verbose)?;

    // Parse the emulation profile
    let emulation = parse_emulation(&args.profile)?;
    debug!("Using emulation profile: {:?}", emulation);

    // Start building the client
    let mut builder = Client::builder().emulation(emulation);

    // Configure proxy if provided
    if let (Some(host), Some(port)) = (&args.proxy_host, &args.proxy_port) {
        debug!("Configuring proxy: {}:{}", host, port);
        
        let proxy_url = if let (Some(username), Some(password)) = (&args.proxy_username, &args.proxy_password) {
            format!("http://{}:{}@{}:{}", username, password, host, port)
        } else {
            format!("http://{}:{}", host, port)
        };
        
        let proxy_config = Proxy::all(proxy_url)?;
        builder = builder.proxy(proxy_config);
        debug!("Proxy configured successfully");
    }

    // Build the client
    let client = builder.build()?;
    debug!("Client built successfully");

    // Create the request based on method
    let mut request = match args.method {
        HttpMethod::GET => client.get(&args.url.0),
        HttpMethod::POST => client.post(&args.url.0),
        HttpMethod::PUT => client.put(&args.url.0),
        HttpMethod::DELETE => client.delete(&args.url.0),
        HttpMethod::HEAD => client.head(&args.url.0),
    };

    // Add request body if provided and method supports it
    if let Some(body) = args.body {
        if matches!(args.method, HttpMethod::GET | HttpMethod::HEAD) {
            return Err("Request body is not supported for GET and HEAD methods".into());
        }
        request = request.body(body);
        debug!("Added request body");
    }
    
    // Log detailed request information
    info!("=== Request Details ===");
    info!("URL: {}", args.url.0);
    
    // Parse and log query parameters if present
    if let Ok(url) = url::Url::parse(&args.url.0) {
        if let Some(query) = url.query() {
            info!("Query String: {}", query);
            info!("Query Parameters:");
            for (key, value) in url.query_pairs() {
                info!("  {}: {}", key, value);
            }
        }
    }
    
    info!("Method: {}", args.method);
    info!("Emulation Profile: {:?}", emulation);
    
    // Log all request headers
    let mut request_headers = Vec::new();
    
    // Add cookies if provided
    let mut cookie_header = String::new();

    // Handle single cookie
    if let Some(cookie) = args.cookie {
        debug!("Adding single cookie: {}", cookie.0);
        cookie_header.push_str(&cookie.0);
    }

    // Handle multiple cookies
    if let Some(cookies) = args.cookies {
        debug!("Adding multiple cookies: {}", cookies.0);
        if !cookie_header.is_empty() {
            cookie_header.push_str("; ");
        }
        cookie_header.push_str(&cookies.0);
    }

    // Add cookie header if we have any cookies
    if !cookie_header.is_empty() {
        request = request.header(header::COOKIE, cookie_header.clone());
        request_headers.push(("Cookie".to_string(), cookie_header));
    }

    // Add custom headers
    for header in args.headers {
        debug!("Adding custom header: {}", header.0);
        let (name, value) = parse_header(&header.0)?;
        request = request.header(&name, &value);
        request_headers.push((name, value));
    }

    // Log all headers
    info!("=== Request Headers ===");
    for (name, value) in &request_headers {
        info!("{}: {}", name, value);
    }

    // Send the request
    debug!("Sending request...");
    let resp = request.send().await?;
    debug!("Received response with status: {}", resp.status());
    
    // Extract all information from response before consuming it
    let status = resp.status().as_u16();
    let content_type = resp.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    
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

    // Log response details
    info!("=== Response Details ===");
    info!("Status: {}", status);
    info!("=== Response Headers ===");
    for (name, value) in &headers {
        info!("{}: {}", name, value);
    }

    // Get the body (this consumes the response)
    let body = resp.text().await?;
    debug!("Response body length: {} bytes", body.len());

    // Log response body
    info!("=== Response Body ===");
    
    // Check if response is JSON and pretty print if it is
    if content_type.contains("json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            info!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            info!("{}", body);
        }
    } else {
        info!("{}", body);
    }

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