use super::*;

macro_rules! tls_options {
    (@build $builder:expr) => {
        $builder.build().into()
    };

    (1) => {
        tls_options!(@build ChromeTlsConfig::builder())
    };
    (2) => {
        tls_options!(@build ChromeTlsConfig::builder().enable_ech_grease(true))
    };
    (3) => {
        tls_options!(@build ChromeTlsConfig::builder().permute_extensions(true))
    };
    (4) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true))
    };
    (5) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true))
    };
    (6, $curves:expr) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves))
    };
    (7, $curves:expr) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves)
            .alps_use_new_codepoint(true))
    };
    (8, $curves:expr, $sigalgs:expr) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves)
            .sigalgs_list($sigalgs)
            .alps_use_new_codepoint(true))
    };
    (9, $curves:expr) => {
        tls_options!(9, $curves, shuffled_trust_anchors(crate::rand::fast_random))
    };
    (9, $curves:expr, $trust_anchors:expr) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves)
            .sigalgs_list(NEW_SIGALGS_LIST)
            .trust_anchors($trust_anchors)
            .grease_sigalgs_enabled(true)
            .alps_use_new_codepoint(true))
    };
}

pub const CURVES_1: &str = join!(":", "X25519", "P-256", "P-384");
pub const CURVES_2: &str = join!(":", "X25519Kyber768Draft00", "X25519", "P-256", "P-384");
pub const CURVES_3: &str = join!(":", "X25519MLKEM768", "X25519", "P-256", "P-384");

pub const CIPHER_LIST: &str = join!(
    ":",
    "TLS_AES_128_GCM_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
    "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
    "TLS_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_RSA_WITH_AES_128_CBC_SHA",
    "TLS_RSA_WITH_AES_256_CBC_SHA"
);

pub const SIGALGS_LIST: &str = join!(
    ":",
    "ecdsa_secp256r1_sha256",
    "rsa_pss_rsae_sha256",
    "rsa_pkcs1_sha256",
    "ecdsa_secp384r1_sha384",
    "rsa_pss_rsae_sha384",
    "rsa_pkcs1_sha384",
    "rsa_pss_rsae_sha512",
    "rsa_pkcs1_sha512"
);

pub const NEW_SIGALGS_LIST: &str = join!(
    ":",
    "mldsa44",
    "mldsa65",
    "mldsa87",
    "ecdsa_secp256r1_sha256",
    "rsa_pss_rsae_sha256",
    "rsa_pkcs1_sha256",
    "ecdsa_secp384r1_sha384",
    "rsa_pss_rsae_sha384",
    "rsa_pkcs1_sha384",
    "rsa_pss_rsae_sha512",
    "rsa_pkcs1_sha512"
);

pub const CERTIFICATE_COMPRESSORS: &[&'static dyn CertificateCompressor] = &[&BrotliCompressor];

// Encoded IDs and wreq's Chromium root store come from the same root set.
#[cfg(feature = "emulation-chromium-pki")]
pub(super) const CHROME_TRUST_ANCHORS: &[u8] = &chromium_roots::encoded_trust_anchor_ids();

// Chrome 152/153 iterate a salted flat_hash_set held by SSLClientContext, rather than
// the root store's source order. Generate an order once per native configuration;
// connections using that client configuration retain the order.
// https://chromium.googlesource.com/chromium/src/+/f7b831a4e249fbd98eae4b391c69f70300a849d1/net/ssl/ssl_config_service.cc
#[cfg(feature = "emulation-chromium-pki")]
pub(super) fn shuffled_trust_anchors(mut random: impl FnMut() -> u64) -> Vec<u8> {
    let mut ids = chromium_roots::trust_anchor_ids();
    for index in (1..ids.len()).rev() {
        let bound = (index + 1) as u64;
        let limit = u64::MAX - u64::MAX % bound;
        let mut value = random();
        while value >= limit {
            value = random();
        }
        ids.swap(index, (value % bound) as usize);
    }

    encode_trust_anchors(&ids)
}

#[cfg(not(feature = "emulation-chromium-pki"))]
pub(super) fn shuffled_trust_anchors(_: impl FnMut() -> u64) -> Vec<u8> {
    Vec::new()
}

// Chrome 154 sorts the raw IDs before adding their length prefixes.
// https://chromium.googlesource.com/chromium/src/+/731082f0a26ce4b3976c3d82943092f5d13daf13/net/cert/x509_util.cc
#[cfg(feature = "emulation-chromium-pki")]
pub(super) fn sorted_trust_anchors() -> Vec<u8> {
    let mut ids = chromium_roots::trust_anchor_ids();
    ids.sort_unstable();
    encode_trust_anchors(&ids)
}

#[cfg(not(feature = "emulation-chromium-pki"))]
pub(super) fn sorted_trust_anchors() -> Vec<u8> {
    Vec::new()
}

#[cfg(feature = "emulation-chromium-pki")]
fn encode_trust_anchors(ids: &[&[u8]]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(CHROME_TRUST_ANCHORS.len());
    for id in ids {
        // encoded_trust_anchor_ids() validates these same IDs at compile time:
        // each length is nonzero and fits in one byte.
        encoded.push(id.len() as u8);
        encoded.extend_from_slice(id);
    }
    encoded
}

#[derive(TypedBuilder)]
pub struct ChromeTlsConfig {
    #[builder(default = CURVES_1)]
    curves: &'static str,

    #[builder(default = SIGALGS_LIST)]
    sigalgs_list: &'static str,

    #[builder(default = CIPHER_LIST)]
    cipher_list: &'static str,

    #[builder(default = AlpsProtocol::HTTP2, setter(into))]
    alps_protos: AlpsProtocol,

    #[builder(default = false)]
    alps_use_new_codepoint: bool,

    #[builder(default = false, setter(into))]
    enable_ech_grease: bool,

    #[builder(default = false, setter(into))]
    permute_extensions: bool,

    #[builder(default = false, setter(into))]
    pre_shared_key: bool,

    #[builder(default, setter(strip_option))]
    trust_anchors: Option<Vec<u8>>,

    #[builder(default, setter(strip_option))]
    grease_sigalgs_enabled: Option<bool>,
}

impl From<ChromeTlsConfig> for TlsOptions {
    fn from(val: ChromeTlsConfig) -> Self {
        let mut opts = TlsOptions::builder()
            .grease_enabled(true)
            .enable_ocsp_stapling(true)
            .enable_signed_cert_timestamps(true)
            .curves_list(val.curves)
            .sigalgs_list(val.sigalgs_list)
            .cipher_list(val.cipher_list)
            .min_tls_version(TlsVersion::TLS_1_2)
            .max_tls_version(TlsVersion::TLS_1_3)
            .permute_extensions(val.permute_extensions)
            .pre_shared_key(val.pre_shared_key)
            .enable_ech_grease(val.enable_ech_grease)
            .alps_protocols([val.alps_protos])
            .alps_use_new_codepoint(val.alps_use_new_codepoint)
            .aes_hw_override(true)
            .certificate_compressors(CERTIFICATE_COMPRESSORS)
            .build();
        if let Some(trust_anchors) = val.trust_anchors {
            opts.trust_anchors = Some(trust_anchors.into());
        }
        if let Some(enabled) = val.grease_sigalgs_enabled {
            opts.grease_sigalgs_enabled = Some(enabled);
        }
        opts
    }
}

#[cfg(test)]
mod tests {
    use wreq::IntoEmulation;

    #[cfg(feature = "emulation-chromium-pki")]
    use super::*;

    #[test]
    fn chrome_profiles_preserve_version_specific_trust_anchors() {
        for profile in [
            crate::Profile::Chrome152,
            crate::Profile::Chrome153,
            crate::Profile::Chrome154,
        ] {
            let native = profile.into_emulation();
            let cloned = native.clone();
            let ids = native.tls_options.unwrap().trust_anchors.unwrap();
            assert_eq!(ids, cloned.tls_options.unwrap().trust_anchors.unwrap());
            #[cfg(feature = "emulation-chromium-pki")]
            {
                assert_eq!(ids.len(), CHROME_TRUST_ANCHORS.len());
                let decoded = decode_ids(&ids);
                let mut expected = chromium_roots::trust_anchor_ids();
                expected.sort_unstable();
                if profile == crate::Profile::Chrome154 {
                    assert_eq!(decoded, expected);
                }
                let mut actual = decoded;
                actual.sort_unstable();
                assert_eq!(actual, expected);
            }
            #[cfg(not(feature = "emulation-chromium-pki"))]
            assert!(ids.is_empty());
        }
        assert!(
            crate::Profile::Chrome151
                .into_emulation()
                .tls_options
                .unwrap()
                .trust_anchors
                .is_none()
        );
    }

    #[cfg(feature = "emulation-chromium-pki")]
    #[test]
    fn trust_anchor_shuffle_preserves_ids_without_pinning_the_first() {
        // Choosing index zero at every step rotates the entire list left.
        // Rejecting u64::MAX also covers the unbiased sampling boundary.
        let mut samples = std::iter::once(u64::MAX).chain(std::iter::repeat(0));
        let encoded = shuffled_trust_anchors(|| samples.next().unwrap());
        let decoded = decode_ids(&encoded);
        let mut expected = chromium_roots::trust_anchor_ids();
        expected.rotate_left(1);
        assert_eq!(decoded, expected);
        assert_eq!(encoded.len(), CHROME_TRUST_ANCHORS.len());
    }

    #[cfg(feature = "emulation-chromium-pki")]
    fn decode_ids(mut remaining: &[u8]) -> Vec<&[u8]> {
        let mut decoded = Vec::new();
        while let Some((&length, rest)) = remaining.split_first() {
            assert_ne!(length, 0);
            let (id, rest) = rest.split_at(usize::from(length));
            decoded.push(id);
            remaining = rest;
        }
        decoded
    }
}
