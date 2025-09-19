use super::tls_imports::*;

pub const CURVES_1: &str = join!(
    ":",
    "X25519",
    "P-256",
    "P-384",
    "P-521",
    "ffdhe2048",
    "ffdhe3072"
);
pub const CURVES_2: &str = join!(
    ":",
    "X25519MLKEM768",
    "X25519",
    "P-256",
    "P-384",
    "P-521",
    "ffdhe2048",
    "ffdhe3072"
);

pub const CIPHER_LIST_1: &str = join!(
    ":",
    "TLS_AES_128_GCM_SHA256",
    "TLS_CHACHA20_POLY1305_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA",
    "TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA",
    "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
    "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
    "TLS_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_RSA_WITH_AES_128_CBC_SHA",
    "TLS_RSA_WITH_AES_256_CBC_SHA"
);
pub const CIPHER_LIST_2: &str = join!(
    ":",
    "TLS_AES_128_GCM_SHA256",
    "TLS_CHACHA20_POLY1305_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
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
    "ecdsa_secp384r1_sha384",
    "ecdsa_secp521r1_sha512",
    "rsa_pss_rsae_sha256",
    "rsa_pss_rsae_sha384",
    "rsa_pss_rsae_sha512",
    "rsa_pkcs1_sha256",
    "rsa_pkcs1_sha384",
    "rsa_pkcs1_sha512",
    "ecdsa_sha1",
    "rsa_pkcs1_sha1"
);

pub const CERT_COMPRESSION_ALGORITHM: &[CertificateCompressionAlgorithm] = &[
    CertificateCompressionAlgorithm::ZLIB,
    CertificateCompressionAlgorithm::BROTLI,
    CertificateCompressionAlgorithm::ZSTD,
];

pub const DELEGATED_CREDENTIALS: &str = join!(
    ":",
    "ecdsa_secp256r1_sha256",
    "ecdsa_secp384r1_sha384",
    "ecdsa_secp521r1_sha512",
    "ecdsa_sha1"
);

pub const RECORD_SIZE_LIMIT: u16 = 0x4001;

pub const EXTENSION_PERMUTATION_INDICES: &[ExtensionType] = &[
    ExtensionType::SERVER_NAME,
    ExtensionType::EXTENDED_MASTER_SECRET,
    ExtensionType::RENEGOTIATE,
    ExtensionType::SUPPORTED_GROUPS,
    ExtensionType::EC_POINT_FORMATS,
    ExtensionType::SESSION_TICKET,
    ExtensionType::APPLICATION_LAYER_PROTOCOL_NEGOTIATION,
    ExtensionType::STATUS_REQUEST,
    ExtensionType::DELEGATED_CREDENTIAL,
    ExtensionType::CERTIFICATE_TIMESTAMP,
    ExtensionType::KEY_SHARE,
    ExtensionType::SUPPORTED_VERSIONS,
    ExtensionType::SIGNATURE_ALGORITHMS,
    ExtensionType::PSK_KEY_EXCHANGE_MODES,
    ExtensionType::RECORD_SIZE_LIMIT,
    ExtensionType::CERT_COMPRESSION,
    ExtensionType::ENCRYPTED_CLIENT_HELLO,
];

#[derive(TypedBuilder)]
pub struct FirefoxTlsConfig {
    #[builder(default = SIGALGS_LIST)]
    sigalgs_list: &'static str,

    #[builder(setter(into))]
    cipher_list: &'static str,

    #[builder(setter(into))]
    curves_list: &'static str,

    #[builder(default = true)]
    session_ticket: bool,

    #[builder(default = false, setter(into))]
    enable_ech_grease: bool,

    #[builder(default = false, setter(into))]
    enable_signed_cert_timestamps: bool,

    #[builder(default = false, setter(into))]
    pre_shared_key: bool,

    #[builder(default = false, setter(into))]
    psk_skip_session_tickets: bool,

    #[builder(default = DELEGATED_CREDENTIALS, setter(into))]
    delegated_credentials: &'static str,

    #[builder(default = RECORD_SIZE_LIMIT, setter(into))]
    record_size_limit: u16,

    #[builder(default, setter(into))]
    key_shares_limit: Option<u8>,

    #[builder(default = true, setter(into))]
    psk_dhe_ke: bool,

    #[builder(default, setter(into))]
    certificate_compression_algorithms: Option<&'static [CertificateCompressionAlgorithm]>,

    #[builder(default = EXTENSION_PERMUTATION_INDICES, setter(into))]
    extension_permutation: &'static [ExtensionType],
}

impl From<FirefoxTlsConfig> for TlsOptions {
    fn from(val: FirefoxTlsConfig) -> Self {
        let mut builder = TlsOptions::builder()
            .curves_list(val.curves_list)
            .sigalgs_list(val.sigalgs_list)
            .cipher_list(val.cipher_list)
            .session_ticket(val.session_ticket)
            .delegated_credentials(val.delegated_credentials)
            .record_size_limit(val.record_size_limit)
            .enable_ocsp_stapling(true)
            .enable_ech_grease(val.enable_ech_grease)
            .enable_signed_cert_timestamps(val.enable_signed_cert_timestamps)
            .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
            .min_tls_version(TlsVersion::TLS_1_2)
            .max_tls_version(TlsVersion::TLS_1_3)
            .key_shares_limit(val.key_shares_limit)
            .pre_shared_key(val.pre_shared_key)
            .psk_skip_session_ticket(val.psk_skip_session_tickets)
            .psk_dhe_ke(val.psk_dhe_ke)
            .preserve_tls13_cipher_list(true)
            .extension_permutation(val.extension_permutation)
            .aes_hw_override(true)
            .random_aes_hw_override(true);

        if let Some(cert_compression_algorithms) = val.certificate_compression_algorithms {
            builder = builder.certificate_compression_algorithms(cert_compression_algorithms)
        }

        builder.build()
    }
}
