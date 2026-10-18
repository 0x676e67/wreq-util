use wreq::{
    IntoEmulation,
    header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};

use super::{
    order::rank,
    parse::{parse_brands, quote},
    versions::{Os, chrome_full_version},
};

pub(super) const SEC_CH_UA: HeaderName = HeaderName::from_static("sec-ch-ua");
const SEC_CH_UA_MOBILE: HeaderName = HeaderName::from_static("sec-ch-ua-mobile");
const SEC_CH_UA_PLATFORM: HeaderName = HeaderName::from_static("sec-ch-ua-platform");
const SEC_CH_UA_ARCH: HeaderName = HeaderName::from_static("sec-ch-ua-arch");
const SEC_CH_UA_BITNESS: HeaderName = HeaderName::from_static("sec-ch-ua-bitness");
const SEC_CH_UA_FORM_FACTORS: HeaderName = HeaderName::from_static("sec-ch-ua-form-factors");
const SEC_CH_UA_FULL_VERSION: HeaderName = HeaderName::from_static("sec-ch-ua-full-version");
const SEC_CH_UA_FULL_VERSION_LIST: HeaderName =
    HeaderName::from_static("sec-ch-ua-full-version-list");
const SEC_CH_UA_MODEL: HeaderName = HeaderName::from_static("sec-ch-ua-model");
const SEC_CH_UA_PLATFORM_VERSION: HeaderName =
    HeaderName::from_static("sec-ch-ua-platform-version");
const SEC_CH_UA_WOW64: HeaderName = HeaderName::from_static("sec-ch-ua-wow64");

/// Chrome started sending `Sec-CH-UA-Form-Factors` in version 124, which enables the
/// `ClientHintsFormFactors` feature by default.
const FORM_FACTORS_SINCE: u16 = 124;

/// Chrome on Linux reported the kernel version as its platform version before version 144, and
/// reports an empty string since `ReduceUserAgentDataLinuxPlatformVersion` became stable.
const LINUX_EMPTY_PLATFORM_VERSION_SINCE: u16 = 144;

/// The client hints a client can send when a server asks for them.
///
/// Each entry is a request header and the value to send for it. [`ClientHintsLayer`] only sends an
/// entry to origins that named its header in `Accept-CH`.
///
/// Browsers send `Sec-CH-UA`, `Sec-CH-UA-Mobile` and `Sec-CH-UA-Platform` on every request, so
/// those belong in the client's default headers and not here.
///
/// Setters ignore values that aren't printable ASCII.
///
/// # Derived Values
///
/// [`Hints::from_headers`] and [`Hints::from_emulation`] fill in the high-entropy User-Agent hints
/// of a Google Chrome identity on Windows, macOS, Linux or Android. The full version is the one
/// in `User-Agent` if that has one, and otherwise the last stable build of that Chrome milestone
/// on that platform. The brands of `Sec-CH-UA-Full-Version-List` keep the order they have in
/// `Sec-CH-UA`. The device is a fixed one per platform:
///
/// | Platform | Arch    | Bitness | Model       | Platform version                        |
/// |----------|---------|---------|-------------|-----------------------------------------|
/// | Windows  | `"x86"` | `"64"`  | `""`        | `"10.0.0"` (Windows 10)                 |
/// | macOS    | `"arm"` | `"64"`  | `""`        | `"15.6.1"`                              |
/// | Linux    | `"x86"` | `"64"`  | `""`        | `""`, or `"6.8.0"` before Chrome 144    |
/// | Android  | `""`    | `""`    | `"Pixel 6"` | `"15.0.0"`                              |
///
/// Use the setters to describe another device:
///
/// ```ignore
/// let hints = Hints::from_emulation(Emulation::Chrome149)
///     .map(|hints| hints.platform_version("19.0.0"));
/// ```
///
/// [`ClientHintsLayer`]: super::ClientHintsLayer
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hints {
    headers: Vec<(HeaderName, HeaderValue)>,
}

// ===== impl Hints =====

impl Hints {
    /// Creates a new, empty [`Hints`].
    #[inline]
    pub const fn new() -> Self {
        Hints {
            headers: Vec::new(),
        }
    }

    /// Derives the hints of the browser described by a set of request headers.
    ///
    /// Reads `Sec-CH-UA`, `Sec-CH-UA-Mobile`, `Sec-CH-UA-Platform` and `User-Agent`. Returns
    /// `None` unless they describe Google Chrome 100 or newer on Windows, macOS, Linux or Android,
    /// in a version whose stable builds this module knows.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let brands = parse_brands(headers.get(SEC_CH_UA)?.to_str().ok()?)?;
        let major = brands
            .iter()
            .find(|(brand, _)| brand == "Google Chrome")?
            .1
            .parse::<u16>()
            .ok()?;
        let os = parse_os(headers.get(SEC_CH_UA_PLATFORM)?.to_str().ok()?)?;
        let mobile = headers
            .get(SEC_CH_UA_MOBILE)
            .is_some_and(|mobile| mobile.as_bytes() == b"?1");
        // The hints have to name the build the `User-Agent` does, if it still names one.
        let full_version = headers
            .get(USER_AGENT)
            .and_then(|user_agent| user_agent.to_str().ok())
            .and_then(|user_agent| unreduced_version(user_agent, major))
            .map(str::to_owned)
            .or_else(|| chrome_full_version(major, os))?;

        let (arch, bitness, model, platform_version) = device(os, major);

        // Chromium gives its own brands the full version, and the GREASE brand its major
        // version padded with zeros.
        let full_version_list = brands.iter().map(|(brand, version)| {
            if brand == "Chromium" || brand == "Google Chrome" {
                (brand.as_str(), full_version.clone())
            } else {
                (brand.as_str(), format!("{version}.0.0.0"))
            }
        });

        let mut hints = Hints::new()
            .full_version(&full_version)
            .arch(arch)
            .platform_version(platform_version)
            .model(model)
            .bitness(bitness)
            .wow64(false)
            .full_version_list(full_version_list);

        if major >= FORM_FACTORS_SINCE {
            hints = hints.form_factors([if mobile { "Mobile" } else { "Desktop" }]);
        }

        Some(hints)
    }

    /// Derives the hints of the browser an emulation looks like, from its default headers.
    ///
    /// See [`Hints::from_headers`] for when this returns `None`.
    #[inline]
    pub fn from_emulation<E: IntoEmulation>(emulation: E) -> Option<Self> {
        Self::from_headers(&emulation.into_emulation().headers)
    }

    /// Sets `Sec-CH-UA-Arch`, for example `"x86"` or `"arm"`.
    #[inline]
    pub fn arch(self, arch: &str) -> Self {
        self.string(SEC_CH_UA_ARCH, arch)
    }

    /// Sets `Sec-CH-UA-Bitness`, for example `"64"`.
    #[inline]
    pub fn bitness(self, bitness: &str) -> Self {
        self.string(SEC_CH_UA_BITNESS, bitness)
    }

    /// Sets `Sec-CH-UA-Form-Factors`, for example `["Desktop"]`.
    pub fn form_factors<I, S>(self, form_factors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let value = form_factors
            .into_iter()
            .map(|form_factor| quote(form_factor.as_ref()))
            .collect::<Option<Vec<_>>>()
            .map(|form_factors| form_factors.join(", "));
        self.raw(SEC_CH_UA_FORM_FACTORS, value)
    }

    /// Sets `Sec-CH-UA-Full-Version`, the full version of the browser.
    #[inline]
    pub fn full_version(self, version: &str) -> Self {
        self.string(SEC_CH_UA_FULL_VERSION, version)
    }

    /// Sets `Sec-CH-UA-Full-Version-List` from pairs of brand and full version.
    ///
    /// The brands should be the ones in `Sec-CH-UA`, in the same order.
    pub fn full_version_list<I, B, V>(self, brands: I) -> Self
    where
        I: IntoIterator<Item = (B, V)>,
        B: AsRef<str>,
        V: AsRef<str>,
    {
        let value = brands
            .into_iter()
            .map(|(brand, version)| {
                Some(format!(
                    "{};v={}",
                    quote(brand.as_ref())?,
                    quote(version.as_ref())?
                ))
            })
            .collect::<Option<Vec<_>>>()
            .map(|brands| brands.join(", "));
        self.raw(SEC_CH_UA_FULL_VERSION_LIST, value)
    }

    /// Sets `Sec-CH-UA-Model`, the device model. Desktop browsers send an empty string.
    #[inline]
    pub fn model(self, model: &str) -> Self {
        self.string(SEC_CH_UA_MODEL, model)
    }

    /// Sets `Sec-CH-UA-Platform-Version`, the version of the operating system.
    #[inline]
    pub fn platform_version(self, version: &str) -> Self {
        self.string(SEC_CH_UA_PLATFORM_VERSION, version)
    }

    /// Sets `Sec-CH-UA-WoW64`, whether a 32-bit browser runs on 64-bit Windows.
    #[inline]
    pub fn wow64(self, wow64: bool) -> Self {
        let value = HeaderValue::from_static(if wow64 { "?1" } else { "?0" });
        self.header(SEC_CH_UA_WOW64, value)
    }

    /// Sets any other client hint, such as `Sec-CH-Prefers-Color-Scheme` or `Sec-CH-DPR`.
    ///
    /// `name` is the request header and `value` is sent as given.
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        match self.headers.iter_mut().find(|(hint, _)| *hint == name) {
            Some((_, current)) => *current = value,
            None => {
                self.headers.push((name, value));
                // Keep the hints in the order Chrome writes them, and the ones it doesn't know
                // after those.
                self.headers
                    .sort_by_key(|(name, _)| rank(name).unwrap_or(usize::MAX));
            }
        }
        self
    }

    /// Removes a client hint.
    #[inline]
    pub fn remove(mut self, name: &HeaderName) -> Self {
        self.headers.retain(|(hint, _)| hint != name);
        self
    }

    /// Returns the value sent for a client hint.
    #[inline]
    pub fn get(&self, name: &HeaderName) -> Option<&HeaderValue> {
        self.headers
            .iter()
            .find(|(hint, _)| hint == name)
            .map(|(_, value)| value)
    }

    /// Returns the client hints as request headers, in the order Chrome writes them.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.headers.iter().map(|(name, value)| (name, value))
    }

    /// Returns `true` if there are no client hints.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Sets a hint whose value is a structured field string.
    #[inline]
    fn string(self, name: HeaderName, value: &str) -> Self {
        self.raw(name, quote(value))
    }

    /// Sets a hint to a serialized value, or leaves the hints as they are for `None`.
    fn raw(self, name: HeaderName, value: Option<String>) -> Self {
        match value.and_then(|value| HeaderValue::try_from(value).ok()) {
            Some(value) => self.header(name, value),
            None => self,
        }
    }
}

/// Returns the operating system a `Sec-CH-UA-Platform` value names.
fn parse_os(platform: &str) -> Option<Os> {
    match platform.trim().trim_matches('"') {
        "Windows" => Some(Os::Windows),
        "macOS" => Some(Os::MacOS),
        "Linux" => Some(Os::Linux),
        "Android" => Some(Os::Android),
        _ => None,
    }
}

/// Returns the architecture, bitness, model and platform version of the device Chrome is taken
/// to run on.
fn device(os: Os, major: u16) -> (&'static str, &'static str, &'static str, &'static str) {
    match os {
        Os::Windows => ("x86", "64", "", "10.0.0"),
        Os::MacOS => ("arm", "64", "", "15.6.1"),
        Os::Linux if major < LINUX_EMPTY_PLATFORM_VERSION_SINCE => ("x86", "64", "", "6.8.0"),
        Os::Linux => ("x86", "64", "", ""),
        Os::Android => ("", "", "Pixel 6", "15.0.0"),
    }
}

/// Returns the Chrome version of a `User-Agent`, unless it is reduced to the major version.
fn unreduced_version(user_agent: &str, major: u16) -> Option<&str> {
    let version = user_agent.split("Chrome/").nth(1)?.split(' ').next()?;
    let parts = version
        .split('.')
        .map(|part| part.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    let unreduced = matches!(parts[..], [m, minor, build, patch]
        if m == u32::from(major) && (minor, build, patch) != (0, 0, 0));
    unreduced.then_some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        pairs
            .iter()
            .map(|(name, value)| {
                (
                    HeaderName::from_static(name),
                    HeaderValue::from_static(value),
                )
            })
            .collect()
    }

    fn pairs(hints: &Hints) -> Vec<(&str, &str)> {
        hints
            .iter()
            .map(|(name, value)| (name.as_str(), value.to_str().unwrap()))
            .collect()
    }

    // What Chrome 154.0.8037.98 sent on macOS, with the platform version and build replaced by
    // the ones in the table.
    #[test]
    fn derives_what_chrome_sends_on_macos() {
        let hints = Hints::from_headers(&headers(&[
            (
                "sec-ch-ua",
                r#""Chromium";v="154", "Google Chrome";v="154", "Not A(Brand";v="99""#,
            ),
            ("sec-ch-ua-mobile", "?0"),
            ("sec-ch-ua-platform", r#""macOS""#),
        ]))
        .unwrap();

        assert_eq!(
            pairs(&hints),
            [
                ("sec-ch-ua-full-version", r#""154.0.8037.100""#),
                ("sec-ch-ua-arch", r#""arm""#),
                ("sec-ch-ua-platform-version", r#""15.6.1""#),
                ("sec-ch-ua-model", r#""""#),
                ("sec-ch-ua-bitness", r#""64""#),
                ("sec-ch-ua-wow64", "?0"),
                (
                    "sec-ch-ua-full-version-list",
                    r#""Chromium";v="154.0.8037.100", "Google Chrome";v="154.0.8037.100", "Not A(Brand";v="99.0.0.0""#
                ),
                ("sec-ch-ua-form-factors", r#""Desktop""#),
            ]
        );
    }

    #[test]
    fn derives_per_platform_and_version() {
        let derive = |sec_ch_ua, mobile, platform| {
            Hints::from_headers(&headers(&[
                ("sec-ch-ua", sec_ch_ua),
                ("sec-ch-ua-mobile", mobile),
                ("sec-ch-ua-platform", platform),
            ]))
        };
        let chrome_143 = r#""Google Chrome";v="143", "Chromium";v="143", "A Brand";v="24""#;
        let chrome_144 = r#""Not(A:Brand";v="8", "Chromium";v="144", "Google Chrome";v="144""#;
        let chrome_123 = r#""Google Chrome";v="123", "Not:A-Brand";v="8", "Chromium";v="123""#;

        let linux = derive(chrome_143, "?0", r#""Linux""#).unwrap();
        assert_eq!(
            linux.get(&SEC_CH_UA_PLATFORM_VERSION).unwrap(),
            r#""6.8.0""#
        );
        assert_eq!(linux.get(&SEC_CH_UA_ARCH).unwrap(), r#""x86""#);
        assert_eq!(
            linux.get(&SEC_CH_UA_FULL_VERSION_LIST).unwrap(),
            r#""Google Chrome";v="143.0.7499.192", "Chromium";v="143.0.7499.192", "A Brand";v="24.0.0.0""#
        );

        let linux = derive(chrome_144, "?0", r#""Linux""#).unwrap();
        assert_eq!(linux.get(&SEC_CH_UA_PLATFORM_VERSION).unwrap(), r#""""#);

        let windows = derive(chrome_144, "?0", r#""Windows""#).unwrap();
        assert_eq!(
            windows.get(&SEC_CH_UA_PLATFORM_VERSION).unwrap(),
            r#""10.0.0""#
        );
        assert_eq!(
            windows.get(&SEC_CH_UA_FULL_VERSION).unwrap(),
            r#""144.0.7559.135""#
        );

        let android = derive(chrome_144, "?1", r#""Android""#).unwrap();
        assert_eq!(android.get(&SEC_CH_UA_ARCH).unwrap(), r#""""#);
        assert_eq!(android.get(&SEC_CH_UA_BITNESS).unwrap(), r#""""#);
        assert_eq!(android.get(&SEC_CH_UA_MODEL).unwrap(), r#""Pixel 6""#);
        assert_eq!(android.get(&SEC_CH_UA_FORM_FACTORS).unwrap(), r#""Mobile""#);

        // Chrome didn't send form factors before version 124.
        let old = derive(chrome_123, "?0", r#""Windows""#).unwrap();
        assert_eq!(old.get(&SEC_CH_UA_FORM_FACTORS), None);
    }

    #[test]
    fn derives_the_full_version_the_user_agent_names() {
        let derive = |user_agent| {
            Hints::from_headers(&headers(&[
                (
                    "sec-ch-ua",
                    r#""Not A;Brand";v="99", "Chromium";v="100", "Google Chrome";v="100""#,
                ),
                ("sec-ch-ua-mobile", "?0"),
                ("sec-ch-ua-platform", r#""Windows""#),
                ("user-agent", user_agent),
            ]))
            .unwrap()
        };

        let unreduced = derive("Mozilla/5.0 (Windows NT 10.0) Chrome/100.0.4896.75 Safari/537.36");
        assert_eq!(
            unreduced.get(&SEC_CH_UA_FULL_VERSION).unwrap(),
            r#""100.0.4896.75""#
        );
        assert_eq!(
            unreduced.get(&SEC_CH_UA_FULL_VERSION_LIST).unwrap(),
            r#""Not A;Brand";v="99.0.0.0", "Chromium";v="100.0.4896.75", "Google Chrome";v="100.0.4896.75""#
        );

        // A reduced version, or one of another milestone, says nothing about the build.
        for user_agent in [
            "Mozilla/5.0 (Windows NT 10.0) Chrome/100.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0) Chrome/99.0.4844.84 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0) Chrome/100.0.4896 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0) Safari/537.36",
        ] {
            assert_eq!(
                derive(user_agent).get(&SEC_CH_UA_FULL_VERSION).unwrap(),
                r#""100.0.4896.127""#,
                "{user_agent}"
            );
        }
    }

    #[test]
    fn derives_nothing_for_other_browsers() {
        let derive = |sec_ch_ua, platform| {
            Hints::from_headers(&headers(&[
                ("sec-ch-ua", sec_ch_ua),
                ("sec-ch-ua-mobile", "?0"),
                ("sec-ch-ua-platform", platform),
            ]))
        };
        let chrome = r#""Chromium";v="154", "Google Chrome";v="154", "Not A(Brand";v="99""#;
        let edge = r#""Chromium";v="143", "Microsoft Edge";v="143", "Not_A Brand";v="99""#;
        let unknown = r#""Chromium";v="999", "Google Chrome";v="999", "Not A(Brand";v="99""#;

        assert_eq!(derive(edge, r#""Windows""#), None);
        assert_eq!(derive(chrome, r#""iOS""#), None);
        assert_eq!(derive(unknown, r#""Windows""#), None);
        assert_eq!(
            Hints::from_headers(&headers(&[("user-agent", "Mozilla/5.0")])),
            None
        );
    }

    #[test]
    fn setters_replace_and_keep_chrome_order() {
        let color_scheme = HeaderName::from_static("sec-ch-prefers-color-scheme");
        let custom = HeaderName::from_static("x-custom-hint");
        let hints = Hints::new()
            .header(custom.clone(), HeaderValue::from_static("1"))
            .header(color_scheme.clone(), HeaderValue::from_static("dark"))
            .wow64(true)
            .model("Pixel \"9\"")
            .arch("x86")
            .arch("arm")
            .header(
                HeaderName::from_static("sec-ch-dpr"),
                HeaderValue::from_static("2"),
            );

        assert_eq!(
            pairs(&hints),
            [
                ("sec-ch-dpr", "2"),
                ("sec-ch-ua-arch", r#""arm""#),
                ("sec-ch-ua-model", r#""Pixel \"9\"""#),
                ("sec-ch-ua-wow64", "?1"),
                ("sec-ch-prefers-color-scheme", "dark"),
                ("x-custom-hint", "1"),
            ]
        );

        let hints = hints.remove(&custom).remove(&color_scheme);
        assert_eq!(hints.get(&custom), None);
        assert_eq!(hints.iter().count(), 4);
        assert!(!hints.is_empty());
        assert!(Hints::new().is_empty());
    }

    #[test]
    fn ignores_values_that_are_not_printable_ascii() {
        assert!(Hints::new().model("Pixel\n6").is_empty());
        assert_eq!(
            Hints::new().model("Pixel 6").model("Pixel\n6"),
            Hints::new().model("Pixel 6")
        );
        assert!(Hints::new().form_factors(["Desktop", "Tablé"]).is_empty());
        assert!(
            Hints::new()
                .full_version_list([("Brand", "1\u{7f}")])
                .is_empty()
        );
    }

    #[cfg(feature = "emulation")]
    #[test]
    fn every_chrome_profile_has_hints() {
        use crate::{Emulation, Platform, Profile};

        let platforms = [
            Platform::Windows,
            Platform::MacOS,
            Platform::Linux,
            Platform::Android,
        ];
        for profile in Profile::VARIANTS {
            let name = format!("{profile:?}");
            let Some(major) = name.strip_prefix("Chrome") else {
                continue;
            };
            for platform in platforms {
                let emulation = Emulation::builder()
                    .profile(*profile)
                    .platform(platform)
                    .build()
                    .into_emulation();
                let hints = Hints::from_headers(&emulation.headers)
                    .unwrap_or_else(|| panic!("no hints for {name} on {platform:?}"));
                let full_version = hints
                    .get(&SEC_CH_UA_FULL_VERSION)
                    .and_then(|version| version.to_str().ok())
                    .unwrap()
                    .trim_matches('"');
                assert!(
                    full_version.starts_with(&format!("{major}.0.")),
                    "{name} on {platform:?} has full version {full_version}"
                );

                // A `User-Agent` either names the same build or none.
                let user_agent = emulation.headers[USER_AGENT].to_str().unwrap();
                assert!(
                    user_agent.contains(&format!("Chrome/{full_version} "))
                        || user_agent.contains(&format!("Chrome/{major}.0.0.0 ")),
                    "{name} on {platform:?} has full version {full_version} and {user_agent}"
                );
            }
        }
    }

    #[cfg(feature = "emulation")]
    #[test]
    fn other_profiles_have_no_hints() {
        use crate::{Emulation, Platform, Profile};

        for profile in [
            Profile::Firefox152,
            Profile::Safari26_4,
            Profile::OkHttp5,
            Profile::Edge148,
            Profile::Opera131,
        ] {
            assert_eq!(Hints::from_emulation(profile), None, "{profile:?}");
        }

        let ios = Emulation::builder()
            .profile(Profile::Chrome154)
            .platform(Platform::IOS)
            .build();
        assert_eq!(Hints::from_emulation(ios), None);

        let no_headers = Emulation::builder()
            .profile(Profile::Chrome154)
            .headers(false)
            .build();
        assert_eq!(Hints::from_emulation(no_headers), None);
    }
}
