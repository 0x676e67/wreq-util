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
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves)
            .sigalgs_list(NEW_SIGALGS_LIST)
            .trust_anchors(CHROME_TRUST_ANCHORS)
            .grease_sigalgs_enabled(true)
            .alps_use_new_codepoint(true))
    };
    (10, $curves:expr) => {
        tls_options!(@build ChromeTlsConfig::builder()
            .permute_extensions(true)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .curves($curves)
            .sigalgs_list(NEW_SIGALGS_LIST)
            .trust_anchors(CHROME_SORTED_TRUST_ANCHORS)
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
#[cfg(not(feature = "emulation-chromium-pki"))]
pub(super) const CHROME_TRUST_ANCHORS: &[u8] = &[];

#[cfg(feature = "emulation-chromium-pki")]
pub(super) const CHROME_SORTED_TRUST_ANCHORS: &[u8] =
    &sorted_trust_anchors::<{ CHROME_TRUST_ANCHORS.len() }>();
#[cfg(not(feature = "emulation-chromium-pki"))]
pub(super) const CHROME_SORTED_TRUST_ANCHORS: &[u8] = &[];

#[cfg(feature = "emulation-chromium-pki")]
const fn sorted_trust_anchors<const N: usize>() -> [u8; N] {
    let mut ids = chromium_roots::trust_anchor_ids();
    let mut i = 1;
    while i < ids.len() {
        let mut j = i;
        while j > 0 && trust_anchor_id_lt(ids[j], ids[j - 1]) {
            ids.swap(j, j - 1);
            j -= 1;
        }
        i += 1;
    }

    let mut encoded = [0; N];
    let mut offset = 0;
    let mut i = 0;
    while i < ids.len() {
        let id = ids[i];
        encoded[offset] = id.len() as u8;
        offset += 1;
        let mut j = 0;
        while j < id.len() {
            encoded[offset] = id[j];
            offset += 1;
            j += 1;
        }
        i += 1;
    }

    assert!(offset == N);
    encoded
}

#[cfg(feature = "emulation-chromium-pki")]
const fn trust_anchor_id_lt(a: &[u8], b: &[u8]) -> bool {
    let mut i = 0;
    while i < a.len() && i < b.len() {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
        i += 1;
    }
    a.len() < b.len()
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
    trust_anchors: Option<&'static [u8]>,

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
