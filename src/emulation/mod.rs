mod device;
#[cfg(feature = "emulation-rand")]
mod rand;

use device::{chrome::*, firefox::*, okhttp::*, opera::*, safari::*};
#[cfg(feature = "emulation-serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "emulation-rand")]
use strum_macros::VariantArray;
use typed_builder::TypedBuilder;
use wreq::{EmulationProvider, EmulationProviderFactory};

macro_rules! define_emulation_enum {
    ($(#[$meta:meta])* $name:ident, $default_variant:ident, $($variant:ident => $rename:expr),*) => {
        $(#[$meta])*
        #[non_exhaustive]
        #[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "emulation-rand", derive(VariantArray))]
        #[cfg_attr(feature = "emulation-serde", derive(Deserialize, Serialize))]
        pub enum $name {
            $(
                #[cfg_attr(feature = "emulation-serde", serde(rename = $rename))]
                $variant,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$default_variant
            }
        }
    };
}

define_emulation_enum!(
    /// Represents different browser versions for impersonation.
    ///
    /// The `Emulation` enum provides variants for different browser versions that can be used
    /// to emulation HTTP requests. Each variant corresponds to a specific browser version.
    ///
    /// # Naming Convention
    ///
    /// The naming convention for the variants follows the pattern `browser_version`, where
    /// `browser` is the name of the browser (e.g., `chrome`, `firefox`, `safari`) and `version`
    /// is the version number. For example, `Chrome100` represents Chrome version 100.
    ///
    /// The serialized names of the variants use underscores to separate the browser name and
    /// version number, following the pattern `browser_version`. For example, `Chrome100` is
    /// serialized as `"chrome_100"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wreq_util::Emulation;
    ///
    /// let emulation = Emulation::Chrome100;
    /// let serialized = serde_json::to_string(&emulation).unwrap();
    /// assert_eq!(serialized, "\"chrome_100\"");
    ///
    /// let deserialized: Emulation = serde_json::from_str(&serialized).unwrap();
    /// assert_eq!(deserialized, Emulation::Chrome100);
    /// ```
    Emulation, Chrome133,
    Chrome100 => "chrome_100",
    Chrome101 => "chrome_101",
    Chrome104 => "chrome_104",
    Chrome105 => "chrome_105",
    Chrome106 => "chrome_106",
    Chrome107 => "chrome_107",
    Chrome108 => "chrome_108",
    Chrome109 => "chrome_109",
    Chrome110 => "chrome_110",
    Chrome114 => "chrome_114",
    Chrome116 => "chrome_116",
    Chrome117 => "chrome_117",
    Chrome118 => "chrome_118",
    Chrome119 => "chrome_119",
    Chrome120 => "chrome_120",
    Chrome123 => "chrome_123",
    Chrome124 => "chrome_124",
    Chrome126 => "chrome_126",
    Chrome127 => "chrome_127",
    Chrome128 => "chrome_128",
    Chrome129 => "chrome_129",
    Chrome130 => "chrome_130",
    Chrome131 => "chrome_131",
    Chrome132 => "chrome_132",
    Chrome133 => "chrome_133",
    Chrome134 => "chrome_134",
    Chrome135 => "chrome_135",
    Chrome136 => "chrome_136",
    Chrome137 => "chrome_137",

    SafariIos17_2 => "safari_ios_17.2",
    SafariIos17_4_1 => "safari_ios_17.4.1",
    SafariIos16_5 => "safari_ios_16.5",
    Safari15_3 => "safari_15.3",
    Safari15_5 => "safari_15.5",
    Safari15_6_1 => "safari_15.6.1",
    Safari16 => "safari_16",
    Safari16_5 => "safari_16.5",
    Safari17_0 => "safari_17.0",
    Safari17_2_1 => "safari_17.2.1",
    Safari17_4_1 => "safari_17.4.1",
    Safari17_5 => "safari_17.5",
    Safari18 => "safari_18",
    SafariIPad18 => "safari_ipad_18",
    Safari18_2 => "safari_18.2",
    SafariIos18_1_1 => "safari_ios_18.1.1",
    Safari18_3 => "safari_18.3",
    Safari18_3_1 => "safari_18.3.1",
    Safari18_5 => "safari_18.5",

    OkHttp3_9 => "okhttp_3.9",
    OkHttp3_11 => "okhttp_3.11",
    OkHttp3_13 => "okhttp_3.13",
    OkHttp3_14 => "okhttp_3.14",
    OkHttp4_9 => "okhttp_4.9",
    OkHttp4_10 => "okhttp_4.10",
    OkHttp4_12 => "okhttp_4.12",
    OkHttp5 => "okhttp_5",

    Edge101 => "edge_101",
    Edge122 => "edge_122",
    Edge127 => "edge_127",
    Edge131 => "edge_131",
    Edge134 => "edge_134",

    Firefox109 => "firefox_109",
    Firefox117 => "firefox_117",
    Firefox128 => "firefox_128",
    Firefox133 => "firefox_133",
    Firefox135 => "firefox_135",
    FirefoxPrivate135 => "firefox_private_135",
    FirefoxAndroid135 => "firefox_android_135",
    Firefox136 => "firefox_136",
    FirefoxPrivate136 => "firefox_private_136",
    Firefox139 => "firefox_139",

    Opera116 => "opera_116",
    Opera117 => "opera_117",
    Opera118 => "opera_118",
    Opera119 => "opera_119"
);

/// ======== Emulation impls ========
impl EmulationProviderFactory for Emulation {
    fn emulation(self) -> EmulationProvider {
        EmulationOption::builder()
            .emulation(self)
            .build()
            .emulation()
    }
}

define_emulation_enum!(
    /// Represents different operating systems for impersonation.
    ///
    /// The `EmulationOS` enum provides variants for different operating systems that can be used
    /// to emulation HTTP requests. Each variant corresponds to a specific operating system.
    ///
    /// # Naming Convention
    ///
    /// The naming convention for the variants follows the pattern `os_name`, where
    /// `os_name` is the name of the operating system (e.g., `windows`, `macos`, `linux`, `android`, `ios`).
    ///
    /// The serialized names of the variants use lowercase letters to represent the operating system names.
    /// For example, `Windows` is serialized as `"windows"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wreq::EmulationOS;
    ///
    /// let emulation_os = EmulationOS::Windows;
    /// let serialized = serde_json::to_string(&emulation_os).unwrap();
    /// assert_eq!(serialized, "\"windows\"");
    ///
    /// let deserialized: EmulationOS = serde_json::from_str(&serialized).unwrap();
    /// assert_eq!(deserialized, EmulationOS::Windows);
    /// ```
    EmulationOS, MacOS,
    Windows => "windows",
    MacOS => "macos",
    Linux => "linux",
    Android => "android",
    IOS => "ios"
);

/// ======== EmulationOS impls ========
impl EmulationOS {
    #[inline]
    const fn platform(&self) -> &'static str {
        match self {
            EmulationOS::MacOS => "\"macOS\"",
            EmulationOS::Linux => "\"Linux\"",
            EmulationOS::Windows => "\"Windows\"",
            EmulationOS::Android => "\"Android\"",
            EmulationOS::IOS => "\"iOS\"",
        }
    }

    #[inline]
    const fn is_mobile(&self) -> bool {
        matches!(self, EmulationOS::Android | EmulationOS::IOS)
    }
}

/// Represents the configuration options for emulating a browser and operating system.
///
/// The `EmulationOption` struct allows you to configure various aspects of browser and OS emulation,
/// including the browser version, operating system, and whether to skip certain features like HTTP/2
/// or headers.
///
/// This struct is typically used to build an `EmulationProvider` that can be applied to HTTP clients
/// for making requests that mimic specific browser and OS configurations.
///
/// # Fields
///
/// - `emulation`: The browser version to emulate. Defaults to `Emulation::default()`.
/// - `emulation_os`: The operating system to emulate. Defaults to `EmulationOS::default()`.
/// - `skip_http2`: Whether to skip HTTP/2 support. Defaults to `false`.
/// - `skip_headers`: Whether to skip adding default headers. Defaults to `false`.
///
/// # Examples
///
/// ```rust
/// use wreq_util::{Emulation, EmulationOS, EmulationOption};
///
/// let emulation_option = EmulationOption::builder()
///     .emulation(Emulation::Chrome134)
///     .emulation_os(EmulationOS::MacOS)
///     .skip_http2(true)
///     .skip_headers(false)
///     .build();
///
/// // Use `emulation_option` to create an EmulationProvider
/// let emulation_provider = emulation_option.emulation();
/// ```
#[derive(Default, Clone, TypedBuilder)]
pub struct EmulationOption {
    /// The browser version to emulation.
    #[builder(default)]
    emulation: Emulation,

    /// The operating system.
    #[builder(default)]
    emulation_os: EmulationOS,

    /// Whether to skip HTTP/2.
    #[builder(default = false)]
    skip_http2: bool,

    /// Whether to skip headers.
    #[builder(default = false)]
    skip_headers: bool,
}

/// ======== EmulationOption impls ========
macro_rules! emulation_match {
    ($ver:expr, $opt:expr, $($variant:pat => $path:expr),+) => {
        match $ver {
            $(
                $variant => $path($opt),
            )+
        }
    }
}

impl EmulationProviderFactory for EmulationOption {
    fn emulation(self) -> EmulationProvider {
        emulation_match!(
            self.emulation,
            self,

            Emulation::Chrome100 => v100::emulation,
            Emulation::Chrome101 => v101::emulation,
            Emulation::Chrome104 => v104::emulation,
            Emulation::Chrome105 => v105::emulation,
            Emulation::Chrome106 => v106::emulation,
            Emulation::Chrome107 => v107::emulation,
            Emulation::Chrome108 => v108::emulation,
            Emulation::Chrome109 => v109::emulation,
            Emulation::Chrome110 => v110::emulation,
            Emulation::Chrome114 => v114::emulation,
            Emulation::Chrome116 => v116::emulation,
            Emulation::Chrome117 => v117::emulation,
            Emulation::Chrome118 => v118::emulation,
            Emulation::Chrome119 => v119::emulation,
            Emulation::Chrome120 => v120::emulation,
            Emulation::Chrome123 => v123::emulation,
            Emulation::Chrome124 => v124::emulation,
            Emulation::Chrome126 => v126::emulation,
            Emulation::Chrome127 => v127::emulation,
            Emulation::Chrome128 => v128::emulation,
            Emulation::Chrome129 => v129::emulation,
            Emulation::Chrome130 => v130::emulation,
            Emulation::Chrome131 => v131::emulation,
            Emulation::Chrome132 => v132::emulation,
            Emulation::Chrome133 => v133::emulation,
            Emulation::Chrome134 => v134::emulation,
            Emulation::Chrome135 => v135::emulation,
            Emulation::Chrome136 => v136::emulation,
            Emulation::Chrome137 => v137::emulation,

            Emulation::SafariIos17_2 => safari_ios_17_2::emulation,
            Emulation::SafariIos17_4_1 => safari_ios_17_4_1::emulation,
            Emulation::SafariIos16_5 => safari_ios_16_5::emulation,
            Emulation::Safari15_3 => safari15_3::emulation,
            Emulation::Safari15_5 => safari15_5::emulation,
            Emulation::Safari15_6_1 => safari15_6_1::emulation,
            Emulation::Safari16 => safari16::emulation,
            Emulation::Safari16_5 => safari16_5::emulation,
            Emulation::Safari17_0 => safari17_0::emulation,
            Emulation::Safari17_2_1 => safari17_2_1::emulation,
            Emulation::Safari17_4_1 => safari17_4_1::emulation,
            Emulation::Safari17_5 => safari17_5::emulation,
            Emulation::Safari18 => safari18::emulation,
            Emulation::SafariIPad18 => safari_ipad_18::emulation,
            Emulation::Safari18_2 => safari18_2::emulation,
            Emulation::SafariIos18_1_1 => safari_ios_18_1_1::emulation,
            Emulation::Safari18_3 => safari18_3::emulation,
            Emulation::Safari18_3_1 => safari18_3_1::emulation,
            Emulation::Safari18_5 => safari18_5::emulation,

            Emulation::OkHttp3_9 => okhttp3_9::emulation,
            Emulation::OkHttp3_11 => okhttp3_11::emulation,
            Emulation::OkHttp3_13 => okhttp3_13::emulation,
            Emulation::OkHttp3_14 => okhttp3_14::emulation,
            Emulation::OkHttp4_9 => okhttp4_9::emulation,
            Emulation::OkHttp4_10 => okhttp4_10::emulation,
            Emulation::OkHttp4_12 => okhttp4_12::emulation,
            Emulation::OkHttp5 => okhttp5::emulation,

            Emulation::Edge101 => edge101::emulation,
            Emulation::Edge122 => edge122::emulation,
            Emulation::Edge127 => edge127::emulation,
            Emulation::Edge131 => edge131::emulation,
            Emulation::Edge134 => edge134::emulation,

            Emulation::Firefox109 => ff109::emulation,
            Emulation::Firefox117 => ff117::emulation,
            Emulation::Firefox128 => ff128::emulation,
            Emulation::Firefox133 => ff133::emulation,
            Emulation::Firefox135 => ff135::emulation,
            Emulation::FirefoxPrivate135 => ff_private_135::emulation,
            Emulation::FirefoxAndroid135 => ff_android_135::emulation,
            Emulation::Firefox136 => ff136::emulation,
            Emulation::FirefoxPrivate136 => ff_private_136::emulation,
            Emulation::Firefox139 => ff139::emulation,

            Emulation::Opera116 => opera116::emulation,
            Emulation::Opera117 => opera117::emulation,
            Emulation::Opera118 => opera118::emulation,
            Emulation::Opera119 => opera119::emulation
        )
    }
}
