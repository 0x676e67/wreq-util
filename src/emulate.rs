#[macro_use]
mod macros;
pub mod compress;
pub mod profile;

use profile::{chrome::*, firefox::*, okhttp::*, opera::*, safari::*};
#[cfg(feature = "emulation-serde")]
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

define_enum!(
    /// Selects which client profile the request should look like.
    ///
    /// This controls the built-in TLS, HTTP/2, and header presets used for the
    /// request. Variants cover browser-style profiles as well as other clients,
    /// such as OkHttp.
    dispatch,
    Profile, Chrome100,
    Emulation,

    // Chrome versions
    Chrome100 => ("chrome_100", v100::emulation),
    Chrome101 => ("chrome_101", v101::emulation),
    Chrome104 => ("chrome_104", v104::emulation),
    Chrome105 => ("chrome_105", v105::emulation),
    Chrome106 => ("chrome_106", v106::emulation),
    Chrome107 => ("chrome_107", v107::emulation),
    Chrome108 => ("chrome_108", v108::emulation),
    Chrome109 => ("chrome_109", v109::emulation),
    Chrome110 => ("chrome_110", v110::emulation),
    Chrome114 => ("chrome_114", v114::emulation),
    Chrome116 => ("chrome_116", v116::emulation),
    Chrome117 => ("chrome_117", v117::emulation),
    Chrome118 => ("chrome_118", v118::emulation),
    Chrome119 => ("chrome_119", v119::emulation),
    Chrome120 => ("chrome_120", v120::emulation),
    Chrome123 => ("chrome_123", v123::emulation),
    Chrome124 => ("chrome_124", v124::emulation),
    Chrome126 => ("chrome_126", v126::emulation),
    Chrome127 => ("chrome_127", v127::emulation),
    Chrome128 => ("chrome_128", v128::emulation),
    Chrome129 => ("chrome_129", v129::emulation),
    Chrome130 => ("chrome_130", v130::emulation),
    Chrome131 => ("chrome_131", v131::emulation),
    Chrome132 => ("chrome_132", v132::emulation),
    Chrome133 => ("chrome_133", v133::emulation),
    Chrome134 => ("chrome_134", v134::emulation),
    Chrome135 => ("chrome_135", v135::emulation),
    Chrome136 => ("chrome_136", v136::emulation),
    Chrome137 => ("chrome_137", v137::emulation),
    Chrome138 => ("chrome_138", v138::emulation),
    Chrome139 => ("chrome_139", v139::emulation),
    Chrome140 => ("chrome_140", v140::emulation),
    Chrome141 => ("chrome_141", v141::emulation),
    Chrome142 => ("chrome_142", v142::emulation),
    Chrome143 => ("chrome_143", v143::emulation),
    Chrome144 => ("chrome_144", v144::emulation),
    Chrome145 => ("chrome_145", v145::emulation),
    Chrome146 => ("chrome_146", v146::emulation),
    Chrome147 => ("chrome_147", v147::emulation),
    Chrome148 => ("chrome_148", v148::emulation),
    Chrome149 => ("chrome_149", v149::emulation),

    // Edge versions
    Edge101 => ("edge_101", edge101::emulation),
    Edge122 => ("edge_122", edge122::emulation),
    Edge127 => ("edge_127", edge127::emulation),
    Edge131 => ("edge_131", edge131::emulation),
    Edge134 => ("edge_134", edge134::emulation),
    Edge135 => ("edge_135", edge135::emulation),
    Edge136 => ("edge_136", edge136::emulation),
    Edge137 => ("edge_137", edge137::emulation),
    Edge138 => ("edge_138", edge138::emulation),
    Edge139 => ("edge_139", edge139::emulation),
    Edge140 => ("edge_140", edge140::emulation),
    Edge141 => ("edge_141", edge141::emulation),
    Edge142 => ("edge_142", edge142::emulation),
    Edge143 => ("edge_143", edge143::emulation),
    Edge144 => ("edge_144", edge144::emulation),
    Edge145 => ("edge_145", edge145::emulation),
    Edge146 => ("edge_146", edge146::emulation),
    Edge147 => ("edge_147", edge147::emulation),
    Edge148 => ("edge_148", edge148::emulation),

    // Opera versions
    Opera116 => ("opera_116", opera116::emulation),
    Opera117 => ("opera_117", opera117::emulation),
    Opera118 => ("opera_118", opera118::emulation),
    Opera119 => ("opera_119", opera119::emulation),
    Opera120 => ("opera_120", opera120::emulation),
    Opera121 => ("opera_121", opera121::emulation),
    Opera122 => ("opera_122", opera122::emulation),
    Opera123 => ("opera_123", opera123::emulation),
    Opera124 => ("opera_124", opera124::emulation),
    Opera125 => ("opera_125", opera125::emulation),
    Opera126 => ("opera_126", opera126::emulation),
    Opera127 => ("opera_127", opera127::emulation),
    Opera128 => ("opera_128", opera128::emulation),
    Opera129 => ("opera_129", opera129::emulation),
    Opera130 => ("opera_130", opera130::emulation),
    Opera131 => ("opera_131", opera131::emulation),

    // Firefox versions
    Firefox109 => ("firefox_109", ff109::emulation),
    Firefox117 => ("firefox_117", ff117::emulation),
    Firefox128 => ("firefox_128", ff128::emulation),
    Firefox133 => ("firefox_133", ff133::emulation),
    Firefox135 => ("firefox_135", ff135::emulation),
    FirefoxPrivate135 => ("firefox_private_135", ff_private_135::emulation),
    FirefoxAndroid135 => ("firefox_android_135", ff_android_135::emulation),
    Firefox136 => ("firefox_136", ff136::emulation),
    FirefoxPrivate136 => ("firefox_private_136", ff_private_136::emulation),
    Firefox139 => ("firefox_139", ff139::emulation),
    Firefox142 => ("firefox_142", ff142::emulation),
    Firefox143 => ("firefox_143", ff143::emulation),
    Firefox144 => ("firefox_144", ff144::emulation),
    Firefox145 => ("firefox_145", ff145::emulation),
    Firefox146 => ("firefox_146", ff146::emulation),
    Firefox147 => ("firefox_147", ff147::emulation),
    Firefox148 => ("firefox_148", ff148::emulation),
    Firefox149 => ("firefox_149", ff149::emulation),
    Firefox150 => ("firefox_150", ff150::emulation),
    Firefox151 => ("firefox_151", ff151::emulation),

    // Safari versions
    SafariIos17_2 => ("safari_ios_17.2", safari_ios_17_2::emulation),
    SafariIos17_4_1 => ("safari_ios_17.4.1", safari_ios_17_4_1::emulation),
    SafariIos16_5 => ("safari_ios_16.5", safari_ios_16_5::emulation),
    Safari15_3 => ("safari_15.3", safari15_3::emulation),
    Safari15_5 => ("safari_15.5", safari15_5::emulation),
    Safari15_6_1 => ("safari_15.6.1", safari15_6_1::emulation),
    Safari16 => ("safari_16", safari16::emulation),
    Safari16_5 => ("safari_16.5", safari16_5::emulation),
    Safari17_0 => ("safari_17.0", safari17_0::emulation),
    Safari17_2_1 => ("safari_17.2.1", safari17_2_1::emulation),
    Safari17_4_1 => ("safari_17.4.1", safari17_4_1::emulation),
    Safari17_5 => ("safari_17.5", safari17_5::emulation),
    Safari17_6 => ("safari_17.6", safari17_6::emulation),
    Safari18 => ("safari_18", safari18::emulation),
    SafariIPad18 => ("safari_ipad_18", safari_ipad_18::emulation),
    Safari18_2 => ("safari_18.2", safari18_2::emulation),
    SafariIos18_1_1 => ("safari_ios_18.1.1", safari_ios_18_1_1::emulation),
    Safari18_3 => ("safari_18.3", safari18_3::emulation),
    Safari18_3_1 => ("safari_18.3.1", safari18_3_1::emulation),
    Safari18_5 => ("safari_18.5", safari18_5::emulation),
    Safari26 => ("safari_26", safari26::emulation),
    Safari26_1 => ("safari_26.1", safari26_1::emulation),
    Safari26_2 => ("safari_26.2", safari26_2::emulation),
    Safari26_3 => ("safari_26.3", safari26_3::emulation),
    Safari26_4 => ("safari_26.4", safari26_4::emulation),
    SafariIPad26 => ("safari_ipad_26", safari_ipad_26::emulation),
    SafariIpad26_2 => ("safari_ipad_26.2", safari_ipad_26_2::emulation),
    SafariIos26 => ("safari_ios_26", safari_ios_26::emulation),
    SafariIos26_2 => ("safari_ios_26.2", safari_ios_26_2::emulation),

    // OkHttp versions
    OkHttp3_9 => ("okhttp_3.9", okhttp3_9::emulation),
    OkHttp3_11 => ("okhttp_3.11", okhttp3_11::emulation),
    OkHttp3_13 => ("okhttp_3.13", okhttp3_13::emulation),
    OkHttp3_14 => ("okhttp_3.14", okhttp3_14::emulation),
    OkHttp4_9 => ("okhttp_4.9", okhttp4_9::emulation),
    OkHttp4_10 => ("okhttp_4.10", okhttp4_10::emulation),
    OkHttp4_12 => ("okhttp_4.12", okhttp4_12::emulation),
    OkHttp5 => ("okhttp_5", okhttp5::emulation)

);

impl wreq::IntoEmulation for Profile {
    #[inline]
    fn into_emulation(self) -> wreq::Emulation {
        Emulation::builder().profile(self).build().into_emulation()
    }
}

define_enum!(
    /// Selects which platform the client should look like.
    ///
    /// This mainly affects platform-specific headers and user-agent details.
    /// In most cases you can keep the default unless you need to match a
    /// specific Windows, macOS, Linux, Android, or iOS profile.
    plain,
    Platform, MacOS,
    Windows => "windows",
    MacOS => "macos",
    Linux => "linux",
    Android => "android",
    IOS => "ios"
);

impl Platform {
    #[inline]
    const fn platform(&self) -> &'static str {
        match self {
            Platform::MacOS => "\"macOS\"",
            Platform::Linux => "\"Linux\"",
            Platform::Windows => "\"Windows\"",
            Platform::Android => "\"Android\"",
            Platform::IOS => "\"iOS\"",
        }
    }

    #[inline]
    const fn is_mobile(&self) -> bool {
        matches!(self, Platform::Android | Platform::IOS)
    }
}

/// Represents the configuration options for emulating a client profile and platform.
///
/// The `Emulation` struct allows you to configure various aspects of profile and platform
/// emulation, including the profile, platform, and whether to enable certain features
/// like HTTP/2 or headers.
#[derive(Default, Clone, TypedBuilder)]
pub struct Emulation {
    /// Whether to change the profile (browser/okhttp) information.
    #[builder(default)]
    profile: Profile,

    /// Whether to change the platform (Windows/macOS/Linux/Android/iOS) information.
    #[builder(default)]
    platform: Platform,

    /// Whether to enable HTTP/2.
    #[builder(default = true)]
    http2: bool,

    /// Whether to include default headers.
    #[builder(default = true)]
    headers: bool,
}

impl Emulation {
    /// Returns a random variant of the `Profile` enum.
    ///
    /// # Examples
    ///
    /// ```
    /// use wreq_util::Emulation;
    ///
    /// let random_emulation = Emulation::random();
    /// println!("{:?}", random_emulation);
    /// ```
    pub fn random() -> Emulation {
        let rand = crate::rand::fast_random();
        Emulation::builder()
            .profile(Profile::VARIANTS[(rand as usize) % Profile::VARIANTS.len()])
            .platform(Platform::VARIANTS[((rand >> 32) as usize) % Platform::VARIANTS.len()])
            .build()
    }

    /// Returns a market-share weighted random `Emulation`.
    ///
    /// Unlike [`Emulation::random`], selection is biased toward popular browser
    /// families and their most recent versions, and each profile is only paired
    /// with platforms it ships on.
    ///
    /// Browser family and version weights are derived from StatCounter Global
    /// Stats data retrieved in June 2026:
    ///
    /// Browser market share:
    /// <https://gs.statcounter.com/browser-market-share#monthly-202506-202606>
    ///
    /// Browser version market share:
    /// <https://gs.statcounter.com/browser-version-market-share#monthly-202506-202606>
    ///
    /// StatCounter requests attribution for use of its data. See:
    /// <https://creativecommons.org/licenses/by-sa/3.0/>
    ///
    /// # Examples
    ///
    /// ```
    /// use wreq_util::Emulation;
    ///
    /// let random_emulation = Emulation::weighted_random();
    /// println!("{:?}", random_emulation);
    /// ```
    pub fn weighted_random() -> Emulation {
        use Platform::*;
        use Profile::*;

        struct Class {
            weight: u32,
            platforms: &'static [Platform],
            profiles: &'static [Profile],
        }

        // Weights based on StatCounter June 2026 data.
        //
        // Each weight is the family's share percentage multiplied by 100 and
        // rounded to an integer (e.g. Chrome 71.41% -> 7141, Edge 5.02% -> 502,
        // Firefox 2.35% -> 235, Opera 1.73% -> 173). Only relative magnitudes
        // matter, so the common x100 scale is arbitrary but keeps two decimals
        // of precision without floats.
        //
        // Safari's 14.77% is split by platform using the browser-version data:
        // iPhone 11.96% + iPad 0.44% = 12.40% mobile (-> 1240), leaving the
        // remaining 2.37% for desktop/macOS (-> 237).
        const CLASSES: &[Class] = &[
            Class {
                weight: 7141,
                platforms: &[Windows, MacOS, Linux, Android],
                profiles: &[
                    Chrome149, Chrome148, Chrome147, Chrome146, Chrome145, Chrome144, Chrome143,
                ],
            },
            Class {
                weight: 1240,
                platforms: &[IOS],
                profiles: &[
                    SafariIos26_2,
                    SafariIos26,
                    SafariIpad26_2,
                    SafariIPad26,
                    SafariIos18_1_1,
                    SafariIPad18,
                ],
            },
            Class {
                weight: 502,
                platforms: &[Windows, MacOS],
                profiles: &[Edge148, Edge147, Edge146, Edge145, Edge144, Edge143],
            },
            Class {
                weight: 237,
                platforms: &[MacOS],
                profiles: &[
                    Safari26_4, Safari26_3, Safari26_2, Safari26_1, Safari26, Safari18_5,
                ],
            },
            Class {
                weight: 235,
                platforms: &[Windows, MacOS, Linux],
                profiles: &[
                    Firefox151, Firefox150, Firefox149, Firefox148, Firefox147, Firefox146,
                ],
            },
            Class {
                weight: 173,
                platforms: &[Windows, MacOS, Linux, Android],
                profiles: &[Opera131, Opera130, Opera129, Opera128, Opera127, Opera126],
            },
        ];

        let (r1, r2) = (crate::rand::fast_random(), crate::rand::fast_random());
        let total: u32 = CLASSES.iter().map(|c| c.weight).sum();
        let mut t = (r1 % total as u64) as u32;
        let class = CLASSES
            .iter()
            .find(|c| {
                t = t.checked_sub(c.weight).unwrap_or(u32::MAX);
                t == u32::MAX
            })
            .unwrap_or(&CLASSES[0]);
        let n = class.profiles.len();
        let idx = ((r1 >> 32) as usize % n).min((r2 >> 32) as usize % n);
        Emulation::builder()
            .profile(class.profiles[idx])
            .platform(class.platforms[(r2 as usize) % class.platforms.len()])
            .build()
    }
}

impl wreq::IntoEmulation for Emulation {
    #[inline]
    fn into_emulation(self) -> wreq::Emulation {
        self.profile.match_emulation(self)
    }
}
