# rquest_runner

A command-line HTTP client with browser emulation capabilities. This binary is part of the rquest-util project and provides a CLI interface for making HTTP requests with browser fingerprint emulation.

## Overview

`rquest_runner` allows you to:
- Make HTTP requests while emulating various browser profiles
- Configure proxy settings with authentication
- Manage cookies for your requests
- Get responses in a structured JSON format

## Builds and Artifacts

The binary is automatically built for multiple platforms:

- Windows (x86_64-pc-windows-msvc): `rquest_runner.exe`
- Linux (x86_64-unknown-linux-gnu): `rquest_runner`

Each build:
1. Compiles with all features enabled (`cli`, `emulation`, `gzip`, `brotli`, `deflate`, `zstd`, `rquest/full`)
2. Runs a test against Cloudflare's trace endpoint
3. Generates artifacts with timestamp (format: `rquest-YYYYMMDD-HHMMSS-{platform}`)
4. Creates a test matrix showing build status and test results

### Build Artifacts

Each release includes:
- The compiled binary
- Test output from Cloudflare trace
- Test matrix summary

Artifacts are retained for 30 days and include build status and test results for each platform.

## Usage

```bash
rquest_runner [OPTIONS] --profile <PROFILE> --method <METHOD> --url <URL>
```

### Required Arguments

- `-P, --profile <PROFILE>`: Browser profile to emulate (e.g., Chrome136, Firefox136, Edge134)
- `-m, --method <METHOD>`: HTTP method (GET, POST, PUT, DELETE, HEAD)
- `-u, --url <URL>`: Target URL to send the request to

### Optional Arguments

- `-x, --proxy <PROXY>`: Proxy configuration in format `ip:port:username:password`
- `-c, --cookie <COOKIE>`: Single cookie in format `name=value`
- `-C, --cookies <COOKIES>`: Multiple cookies in format `"name1=value1; name2=value2"`

## Examples

### Basic Request
```bash
rquest_runner -P Chrome136 -m GET -u https://example.com
```

### Using a Proxy
```bash
rquest_runner -P Chrome136 -m GET -u https://example.com -x 127.0.0.1:8080:user:pass
```

### Setting Cookies
```bash
# Single cookie
rquest_runner -P Chrome136 -m GET -u https://example.com -c "session=123"

# Multiple cookies
rquest_runner -P Chrome136 -m GET -u https://example.com -C "user=john; theme=dark"
```

## Response Format

Responses are returned in JSON format:

```json
{
  "response_code": 200,
  "headers": [
    ["header-name", "header-value"],
    ...
  ],
  "body": "response body content"
}
```

## Supported Browser Profiles

The binary supports emulation of various browser versions:

### Chrome
Chrome versions 100-136

### Firefox
- Standard: Firefox109, Firefox117, Firefox128, Firefox133-136
- Private Mode: FirefoxPrivate135-136
- Android: FirefoxAndroid135

### Edge
Edge101, Edge122, Edge127, Edge131, Edge134

### Safari
- Desktop: Safari15.3-18.3.1, Safari16-18
- Mobile: SafariIos16.5, SafariIos17.2, SafariIos17.4.1
- iPad: SafariIPad18

### OkHttp
OkHttp versions 3.9-5.0

## Error Messages

The binary provides clear error messages for:
- Invalid browser profile selections
- Malformed proxy configurations
- Invalid cookie formats
- Network errors
- Unsupported HTTP methods

## Build Features

The binary is compiled with the following features enabled:
- `cli`: Command-line interface support
- `emulation`: Browser profile emulation
- `gzip`: Gzip compression support
- `brotli`: Brotli compression support
- `deflate`: Deflate compression support
- `zstd`: Zstandard compression support
- `rquest/full`: Full rquest library features 