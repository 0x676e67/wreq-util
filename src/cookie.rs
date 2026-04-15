use std::{fs, io, ops::Deref, path::Path};

use cookie::Cookie;
use http::{Uri, Version, header::HeaderValue, uri::Scheme};
use wreq::cookie::{CookieStore, Cookies, Jar as WreqJar};

#[derive(Debug, Default)]
pub struct Jar(WreqJar);

impl Deref for Jar {
    type Target = WreqJar;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CookieStore for Jar {
    #[inline]
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, uri: &Uri) {
        self.0.set_cookies(cookie_headers, uri)
    }

    #[inline]
    fn cookies(&self, uri: &Uri, version: Version) -> Cookies {
        self.0.cookies(uri, version)
    }
}

impl Jar {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let cookies = self
            .0
            .get_all()
            .map(|cookie| {
                let raw: Cookie<'static> = cookie.into();
                raw.to_string()
            })
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&cookies)
    }

    pub fn load_json(&self, json: &str) -> io::Result<usize> {
        let list: Vec<String> = serde_json::from_str(json)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let mut loaded = 0usize;

        for set_cookie in list {
            let cookie = Cookie::parse(set_cookie)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
                .into_owned();

            if let Some(uri) = uri(&cookie) {
                let name = cookie.name().to_owned();
                let value = cookie.value().to_owned();
                self.0.add(cookie, &uri);

                if self
                    .0
                    .get(&name, &uri)
                    .is_some_and(|stored| stored.value() == value)
                {
                    loaded += 1;
                }
            }
        }

        Ok(loaded)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let json = self
            .to_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn load_from_file(&self, path: impl AsRef<Path>) -> io::Result<usize> {
        let content = fs::read_to_string(path)?;
        self.load_json(&content)
    }
}

fn uri(cookie: &Cookie<'_>) -> Option<Uri> {
    let host = cookie.domain()?.trim_start_matches('.').trim();
    if host.is_empty() {
        return None;
    }

    Uri::builder()
        .scheme(
            cookie
                .secure()
                .unwrap_or(false)
                .then_some(Scheme::HTTPS)
                .unwrap_or(Scheme::HTTP),
        )
        .authority(host)
        .path_and_query(cookie.path().filter(|path| !path.is_empty()).unwrap_or("/"))
        .build()
        .ok()
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn uri(value: &'static str) -> Uri {
        Uri::from_static(value)
    }

    fn cookie_header_value(cookies: Cookies) -> Option<String> {
        match cookies {
            Cookies::Compressed(value) => Some(value.to_str().unwrap().to_owned()),
            Cookies::Uncompressed(values) => Some(
                values
                    .into_iter()
                    .map(|value| value.to_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
            Cookies::Empty => None,
            _ => None,
        }
    }

    fn temp_file_path(prefix: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("wreq-util-{prefix}-{suffix}.json"))
    }

    #[test]
    fn json_roundtrip_restores_cookies() {
        let jar = Jar::default();
        let source_uri = uri("https://example.com/account/login");
        jar.add(
            "sid=abc; Domain=example.com; Path=/account; Secure; HttpOnly",
            &source_uri,
        );

        let json = jar.to_json().unwrap();
        let cookies: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(cookies.len(), 1);
        assert!(cookies[0].contains("sid=abc"));
        assert!(cookies[0].contains("Domain=example.com"));
        assert!(cookies[0].contains("Path=/account"));

        let restored = Jar::default();
        let loaded = restored.load_json(&json).unwrap();
        assert_eq!(loaded, 1);

        let request_uri = uri("https://example.com/account/profile");
        let header = cookie_header_value(restored.cookies(&request_uri, Version::HTTP_11)).unwrap();
        assert_eq!(header, "sid=abc");
    }

    #[test]
    fn json_export_includes_identity_attributes() {
        let jar = Jar::default();
        let source_uri = uri("https://example.com/account/login");
        jar.add("theme=dark; Secure", &source_uri);

        let json = jar.to_json().unwrap();
        let cookies: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(cookies.len(), 1);
        assert!(cookies[0].contains("theme=dark"));
        assert!(cookies[0].contains("Domain=example.com"));
        assert!(cookies[0].contains("Path=/account"));
        assert!(cookies[0].contains("Secure"));
    }

    #[test]
    fn load_json_skips_expired_cookies() {
        let json = serde_json::to_string(&vec![
            "alive=yes; Domain=example.com; Path=/; HttpOnly",
            "expired=no; Domain=example.com; Path=/; Expires=Wed, 21 Oct 2015 07:28:00 GMT",
        ])
        .unwrap();

        let jar = Jar::default();
        let loaded = jar.load_json(&json).unwrap();
        assert_eq!(loaded, 1);

        let request_uri = uri("http://example.com/");
        let header = cookie_header_value(jar.cookies(&request_uri, Version::HTTP_11)).unwrap();
        assert_eq!(header, "alive=yes");
        assert!(jar.to_json().unwrap().contains("alive=yes"));
        assert!(!jar.to_json().unwrap().contains("expired=no"));
    }

    #[test]
    fn save_and_load_file_roundtrip() {
        let path = temp_file_path("cookie-store");
        let save_result = {
            let jar = Jar::default();
            let source_uri = uri("http://example.com/");
            jar.add("theme=dark; Domain=example.com; Path=/", &source_uri);
            jar.save_to_file(&path)
        };

        assert!(save_result.is_ok());

        let restored = Jar::default();
        let loaded = restored.load_from_file(&path).unwrap();
        assert_eq!(loaded, 1);

        let request_uri = uri("http://example.com/settings");
        let header = cookie_header_value(restored.cookies(&request_uri, Version::HTTP_11)).unwrap();
        assert_eq!(header, "theme=dark");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn subdomain_and_path_are_distinct() {
        let jar = Jar::default();
        // Same name, different paths
        jar.add(
            "id=1; Domain=example.com; Path=/a",
            &uri("http://example.com/a"),
        );
        jar.add(
            "id=2; Domain=example.com; Path=/b",
            &uri("http://example.com/b"),
        );
        // Main domain and subdomain
        jar.add(
            "id=3; Domain=example.com; Path=/",
            &uri("http://example.com/"),
        );
        jar.add(
            "id=4; Domain=sub.example.com; Path=/",
            &uri("http://sub.example.com/"),
        );

        let json = jar.to_json().unwrap();
        let restored = Jar::default();
        restored.load_json(&json).unwrap();

        // Path distinction
        let h_a =
            cookie_header_value(restored.cookies(&uri("http://example.com/a"), Version::HTTP_11))
                .unwrap();
        let h_b =
            cookie_header_value(restored.cookies(&uri("http://example.com/b"), Version::HTTP_11))
                .unwrap();
        assert!(h_a.contains("id=1"));
        assert!(!h_a.contains("id=2"));
        assert!(h_b.contains("id=2"));
        assert!(!h_b.contains("id=1"));

        // Subdomain distinction
        let h_main =
            cookie_header_value(restored.cookies(&uri("http://example.com/"), Version::HTTP_11))
                .unwrap();
        let h_sub = cookie_header_value(
            restored.cookies(&uri("http://sub.example.com/"), Version::HTTP_11),
        )
        .unwrap();
        assert!(h_main.contains("id=3"));
        assert!(!h_main.contains("id=4"));
        assert!(h_sub.contains("id=4"));
        // Subdomain requests may also include parent domain cookies (id=3)
    }

    #[test]
    fn overwrite_same_name_domain_path() {
        let jar = Jar::default();
        let u = uri("http://example.com/a");
        jar.add("sid=first; Domain=example.com; Path=/a", &u);
        jar.add("sid=second; Domain=example.com; Path=/a", &u);

        let json = jar.to_json().unwrap();
        let restored = Jar::default();
        restored.load_json(&json).unwrap();

        let h = cookie_header_value(restored.cookies(&u, Version::HTTP_11)).unwrap();
        assert!(h.contains("sid=second"));
        assert!(!h.contains("sid=first"));
    }
}
