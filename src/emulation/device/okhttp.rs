use super::*;

macro_rules! mod_generator {
    ($mod_name:ident, $cipher:expr, $ua:expr) => {
        pub(crate) mod $mod_name {
            use super::*;
            pub fn emulation(option: EmulationOption) -> Emulation {
                build_emulation(option, $cipher, $ua)
            }
        }
    };
}

const CURVES: &str = join!(":", "X25519", "P-256", "P-384");

const SIGALGS_LIST: &str = join!(
    ":",
    "ecdsa_secp256r1_sha256",
    "rsa_pss_rsae_sha256",
    "rsa_pkcs1_sha256",
    "ecdsa_secp384r1_sha384",
    "rsa_pss_rsae_sha384",
    "rsa_pkcs1_sha384",
    "rsa_pss_rsae_sha512",
    "rsa_pkcs1_sha512",
    "rsa_pkcs1_sha1"
);

const CIPHER_LIST: &str = join!(
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
    "TLS_RSA_WITH_AES_256_CBC_SHA",
    "TLS_RSA_WITH_3DES_EDE_CBC_SHA"
);

#[derive(TypedBuilder)]
struct OkHttpTlsConfig {
    #[builder(default = CURVES)]
    curves: &'static str,

    #[builder(default = SIGALGS_LIST)]
    sigalgs_list: &'static str,

    cipher_list: &'static str,
}

impl From<OkHttpTlsConfig> for TlsOptions {
    fn from(val: OkHttpTlsConfig) -> Self {
        TlsOptions::builder()
            .enable_ocsp_stapling(true)
            .curves_list(val.curves)
            .sigalgs_list(val.sigalgs_list)
            .cipher_list(val.cipher_list)
            .min_tls_version(TlsVersion::TLS_1_2)
            .max_tls_version(TlsVersion::TLS_1_3)
            .aes_hw_override(true)
            .build()
    }
}

fn build_emulation(
    option: EmulationOption,
    cipher_list: &'static str,
    user_agent: &'static str,
) -> Emulation {
    let mut builder = Emulation::builder().tls_options(
        OkHttpTlsConfig::builder()
            .cipher_list(cipher_list)
            .build()
            .into(),
    );

    if !option.skip_http2 {
        let settings_order = SettingsOrder::builder()
            .extend([
                SettingId::HeaderTableSize,
                SettingId::EnablePush,
                SettingId::MaxConcurrentStreams,
                SettingId::InitialWindowSize,
                SettingId::MaxFrameSize,
                SettingId::MaxHeaderListSize,
                SettingId::EnableConnectProtocol,
                SettingId::NoRfc7540Priorities,
            ])
            .build();

        let http2_opts = Http2Options::builder()
            .initial_window_size(6291456)
            .initial_connection_window_size(15728640)
            .max_concurrent_streams(1000)
            .max_header_list_size(262144)
            .header_table_size(65536)
            .headers_stream_dependency(StreamDependency::new(StreamId::zero(), 255, true))
            .headers_pseudo_order(
                PseudoOrder::builder()
                    .extend([
                        PseudoId::Method,
                        PseudoId::Path,
                        PseudoId::Authority,
                        PseudoId::Scheme,
                    ])
                    .build(),
            )
            .settings_order(settings_order)
            .build();

        builder = builder.http2_options(http2_opts);
    }

    if !option.skip_headers {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(USER_AGENT, HeaderValue::from_static(user_agent));
        #[cfg(feature = "emulation-compression")]
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );
        builder = builder.headers(headers);
    }

    builder.build()
}

mod_generator!(
    okhttp3_9,
    join!(
        ":",
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA",
        "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
        "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA",
        "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_RSA_WITH_AES_128_CBC_SHA",
        "TLS_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_3DES_EDE_CBC_SHA"
    ),
    "MaiMemo/4.4.50_639 okhttp/3.9 Android/5.0 Channel/WanDouJia Device/alps+M8+Emulator (armeabi-v7a) Screen/4.44 Resolution/480x800 DId/aa6cde19def3806806d5374c4e5fd617 RAM/0.94 ROM/4.91 Theme/Day"
);

mod_generator!(
    okhttp3_11,
    join!(
        ":",
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
        "TLS_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_3DES_EDE_CBC_SHA"
    ),
    "NRC Audio/2.0.6 (nl.nrc.audio; build:36; Android 12; Sdk:31; Manufacturer:motorola; Model: moto g72) OkHttp/3.11.0"
);

mod_generator!(
    okhttp3_13,
    join!(
        ":",
        "TLS_AES_128_GCM_SHA256",
        "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_AES_128_CCM_SHA256",
        "TLS_AES_256_CCM_8_SHA256",
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
        "TLS_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_3DES_EDE_CBC_SHA"
    ),
    "GM-Android/6.112.2 (240590300; M:Google Pixel 7a; O:34; D:2b045e03986fa6dc) ObsoleteUrlFactory/1.0 OkHttp/3.13.0"
);

mod_generator!(
    okhttp3_14,
    CIPHER_LIST,
    "DS podcast/2.0.1 (be.standaard.audio; build:9; Android 11; Sdk:30; Manufacturer:samsung; Model: SM-A405FN) OkHttp/3.14.0"
);

mod_generator!(
    okhttp4_10,
    CIPHER_LIST,
    "GM-Android/6.112.2 (240590300; M:samsung SM-G781U1; O:33; D:edb34792871638d8) ObsoleteUrlFactory/1.0 OkHttp/4.10.0"
);

mod_generator!(
    okhttp4_9,
    join!(
        ":",
        "TLS_AES_128_GCM_SHA256",
        "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
        "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
        "TLS_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_RSA_WITH_AES_256_GCM_SHA384",
        "TLS_RSA_WITH_AES_128_CBC_SHA",
        "TLS_RSA_WITH_AES_256_CBC_SHA"
    ),
    "GM-Android/6.111.1 (240460200; M:motorola moto g power (2021); O:30; D:76ba9f6628d198c8) ObsoleteUrlFactory/1.0 OkHttp/4.9"
);

mod_generator!(okhttp4_12, CIPHER_LIST, "okhttp/4.12.0");

mod_generator!(
    okhttp5,
    CIPHER_LIST,
    "NRC Audio/2.0.6 (nl.nrc.audio; build:36; Android 14; Sdk:34; Manufacturer:OnePlus; Model: CPH2609) OkHttp/5.0.0-alpha2"
);
