#[macro_use]
mod support;

use wreq_util::Emulation;

// Enabling certain extensions(ECH) will change the length during encryption. This is because TLS
// will automatically use padding to fill the data and add a padding extension. At this time, the
// ja4 fingerprint will change.

test_emulation!(
    test_firefox_109,
    Emulation::Firefox109,
    ["t13d1715h2_5b57614c22b0_3d5424432f57"],
    "73d042072ceabaedacfd45e84dff1020"
);

test_emulation!(
    test_firefox_117,
    Emulation::Firefox117,
    ["t13d1715h2_5b57614c22b0_3d5424432f57"],
    "73d042072ceabaedacfd45e84dff1020"
);

test_emulation!(
    test_firefox_128,
    Emulation::Firefox128,
    ["t13d1513h2_8daaf6152771_748f4c70de1c"],
    "1d8a6f51fd7253d04674593073fc18b0"
);

test_emulation!(
    test_firefox_133,
    Emulation::Firefox133,
    ["t13d1716h2_5b57614c22b0_eeeea6562960"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_135,
    Emulation::Firefox135,
    ["t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_private_135,
    Emulation::FirefoxPrivate135,
    ["t13d1715h2_5b57614c22b0_a54fffd0eb61"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_android_135,
    Emulation::FirefoxAndroid135,
    ["t13d1716h2_5b57614c22b0_eeeea6562960"],
    "41a06cadb1c6385e6d08c8d0dbbea818"
);

test_emulation!(
    test_firefox_136,
    Emulation::Firefox136,
    ["t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_private_136,
    Emulation::FirefoxPrivate136,
    ["t13d1715h2_5b57614c22b0_a54fffd0eb61"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_139,
    Emulation::Firefox139,
    ["t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_142,
    Emulation::Firefox142,
    ["t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_143,
    Emulation::Firefox143,
    ["t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_150,
    Emulation::Firefox150,
    ["t13d1617h2_86a278354501_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_151,
    Emulation::Firefox151,
    ["t13d1617h2_86a278354501_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

test_emulation!(
    test_firefox_152,
    Emulation::Firefox152,
    ["t13d1617h2_86a278354501_3cbfd9057e0d"],
    "6ea73faa8fc5aac76bded7bd238f6433"
);

/// Firefox 147 changed the generated `Accept-Language` q-value: the list used to divide 1.0
/// across its entries, giving `q=0.5` for two, and now decrements by 0.1 per entry, giving
/// `q=0.9`. See Bugzilla 2000765, landed in mozilla-firefox 0e050ae5116d.
///
/// Every profile is listed rather than a range, so adding one is a deliberate choice about
/// which side of 147 it falls on.
#[test]
fn firefox_accept_language_matches_the_profile_version() {
    use wreq::IntoEmulation;
    use wreq_util::{Platform, Profile};

    const PRE_147: &[(&str, Profile)] = &[
        ("Firefox109", Emulation::Firefox109),
        ("Firefox117", Emulation::Firefox117),
        ("Firefox128", Emulation::Firefox128),
        ("Firefox133", Emulation::Firefox133),
        ("Firefox135", Emulation::Firefox135),
        ("FirefoxPrivate135", Emulation::FirefoxPrivate135),
        ("FirefoxAndroid135", Emulation::FirefoxAndroid135),
        ("Firefox136", Emulation::Firefox136),
        ("FirefoxPrivate136", Emulation::FirefoxPrivate136),
        ("Firefox139", Emulation::Firefox139),
        ("Firefox142", Emulation::Firefox142),
        ("Firefox143", Emulation::Firefox143),
        ("Firefox144", Emulation::Firefox144),
        ("Firefox145", Emulation::Firefox145),
        ("Firefox146", Emulation::Firefox146),
    ];
    const POST_147: &[(&str, Profile)] = &[
        ("Firefox147", Emulation::Firefox147),
        ("Firefox148", Emulation::Firefox148),
        ("Firefox149", Emulation::Firefox149),
        ("Firefox150", Emulation::Firefox150),
        ("Firefox151", Emulation::Firefox151),
        ("Firefox152", Emulation::Firefox152),
    ];

    for (profiles, expected) in [(PRE_147, "en-US,en;q=0.5"), (POST_147, "en-US,en;q=0.9")] {
        for &(name, profile) in profiles {
            for &platform in Platform::VARIANTS {
                let native = Emulation::builder()
                    .profile(profile)
                    .platform(platform)
                    .build()
                    .into_emulation();
                assert_eq!(
                    native.headers.get("accept-language").unwrap(),
                    expected,
                    "{name} {platform:?}"
                );
            }
        }
    }
}

#[test]
fn firefox150_and_later_remove_the_ecdsa_aes128_cbc_cipher() {
    use wreq::IntoEmulation;

    let previous = Emulation::Firefox149
        .into_emulation()
        .tls_options
        .unwrap()
        .cipher_list
        .unwrap();
    let removed = "TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA";
    assert_eq!(previous.split(':').count(), 17);
    assert!(previous.split(':').any(|cipher| cipher == removed));
    let expected = previous
        .split(':')
        .filter(|&cipher| cipher != removed)
        .collect::<Vec<_>>()
        .join(":");
    for profile in [
        Emulation::Firefox150,
        Emulation::Firefox151,
        Emulation::Firefox152,
    ] {
        let tls = profile.into_emulation().tls_options.unwrap();
        assert_eq!(tls.cipher_list.unwrap(), expected, "{profile:?}");
    }
}
