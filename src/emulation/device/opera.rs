use super::emulation_imports::*;
use super::*;
use http2::*;
use tls::*;

macro_rules! tls_config {
    (6, $curves:expr) => {
        OperaTlsConfig::builder()
            .curves($curves)
            .permute_extensions(true)
            .pre_shared_key(true)
            .enable_ech_grease(true)
            .build()
    };
}

macro_rules! http2_config {
    (3) => {
        Http2Config::builder()
            .initial_stream_window_size(6291456)
            .initial_connection_window_size(15728640)
            .max_header_list_size(262144)
            .header_table_size(65536)
            .enable_push(false)
            .headers_priority(HEADER_PRIORITY)
            .headers_pseudo_order(HEADERS_PSEUDO_ORDER)
            .settings_order(SETTINGS_ORDER)
            .build()
    };
}

#[inline]
fn header_initializer_with_zstd_priority(
    sec_ch_ua: &'static str,
    ua: &'static str,
    emulation_os: EmulationOS,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    header_chrome_sec_ch_ua!(
        headers,
        sec_ch_ua,
        emulation_os.platform(),
        emulation_os.is_mobile()
    );
    header_chrome_ua!(headers, ua);

    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    #[cfg(all(
        feature = "gzip",
        feature = "deflate",
        feature = "brotli",
        feature = "zstd"
    ))]
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(
        HeaderName::from_static("priority"),
        HeaderValue::from_static("u=0, i"),
    );
    headers
}

mod tls {
    use super::tls_imports::*;

    pub const CURVES: &[SslCurve] = &[
        SslCurve::X25519_MLKEM768,
        SslCurve::X25519,
        SslCurve::SECP256R1,
        SslCurve::SECP384R1,
    ];

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

    pub const CERT_COMPRESSION_ALGORITHM: &[CertCompressionAlgorithm] =
        &[CertCompressionAlgorithm::Brotli];

    #[derive(TypedBuilder)]
    pub struct OperaTlsConfig {
        #[builder(default = CURVES)]
        curves: &'static [SslCurve],

        #[builder(default = SIGALGS_LIST)]
        sigalgs_list: &'static str,

        #[builder(default = CIPHER_LIST)]
        cipher_list: &'static str,

        #[builder(default = AlpsProtos::HTTP2, setter(into))]
        alps_protos: AlpsProtos,

        #[builder(default = false)]
        alps_use_new_codepoint: bool,

        #[builder(default = false, setter(into))]
        enable_ech_grease: bool,

        #[builder(default = false, setter(into))]
        permute_extensions: bool,

        #[builder(default = false, setter(into))]
        pre_shared_key: bool,
    }

    impl From<OperaTlsConfig> for TlsConfig {
        fn from(val: OperaTlsConfig) -> Self {
            TlsConfig::builder()
                .grease_enabled(true)
                .enable_ocsp_stapling(true)
                .enable_signed_cert_timestamps(true)
                .curves(val.curves)
                .sigalgs_list(val.sigalgs_list)
                .cipher_list(val.cipher_list)
                .min_tls_version(TlsVersion::TLS_1_2)
                .max_tls_version(TlsVersion::TLS_1_3)
                .permute_extensions(val.permute_extensions)
                .pre_shared_key(val.pre_shared_key)
                .enable_ech_grease(val.enable_ech_grease)
                .alps_protos(val.alps_protos)
                .alps_use_new_codepoint(val.alps_use_new_codepoint)
                .cert_compression_algorithm(CERT_COMPRESSION_ALGORITHM)
                .build()
        }
    }

    impl From<OperaTlsConfig> for Option<TlsConfig> {
        #[inline(always)]
        fn from(val: OperaTlsConfig) -> Self {
            Some(val.into())
        }
    }
}

mod http2 {
    use super::http2_imports::*;

    pub const HEADER_PRIORITY: (u32, u8, bool) = (0, 255, true);

    pub const HEADERS_PSEUDO_ORDER: [PseudoOrder; 4] = [Method, Authority, Scheme, Path];

    pub const SETTINGS_ORDER: [SettingsOrder; 8] = [
        HeaderTableSize,
        EnablePush,
        MaxConcurrentStreams,
        InitialWindowSize,
        MaxFrameSize,
        MaxHeaderListSize,
        UnknownSetting8,
        UnknownSetting9,
    ];
}

macro_rules! mod_generator {
    (
        $mod_name:ident,
        $tls_config:expr,
        $http2_config:expr,
        $header_initializer:ident,
        [($default_os:ident, $default_sec_ch_ua:tt, $default_ua:tt) $(, ($other_os:ident, $other_sec_ch_ua:tt, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            #[inline(always)]
            pub fn emulation(option: EmulationOption) -> EmulationProvider {
                let default_headers = if !option.skip_headers {
                    #[allow(unreachable_patterns)]
                    let default_headers = match option.emulation_os {
                        $(
                            EmulationOS::$other_os => $header_initializer(
                                $other_sec_ch_ua,
                                $other_ua,
                                option.emulation_os,
                            ),
                        )*
                        _ => $header_initializer(
                            $default_sec_ch_ua,
                            $default_ua,
                            EmulationOS::$default_os,
                        ),
                    };
                    Some(default_headers)
                } else {
                    None
                };

                build_emulation(option, default_headers)
            }

            #[inline(always)]
            pub fn build_emulation(
                option: EmulationOption,
                default_headers: Option<HeaderMap>
            ) -> EmulationProvider {
                EmulationProvider::builder()
                     .tls_config($tls_config)
                    .http2_config(conditional_http2!(option.skip_http2, $http2_config))
                    .default_headers(default_headers)
                    .build()
            }
        }
    };
    (
        $mod_name:ident,
        $build_emulation:expr,
        $header_initializer:ident,
        [($default_os:ident, $default_sec_ch_ua:tt, $default_ua:tt) $(, ($other_os:ident, $other_sec_ch_ua:tt, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            #[inline(always)]
            pub fn emulation(option: EmulationOption) -> EmulationProvider {
                let default_headers = if !option.skip_headers {
                    #[allow(unreachable_patterns)]
                    let default_headers = match option.emulation_os {
                        $(
                            EmulationOS::$other_os => $header_initializer(
                                $other_sec_ch_ua,
                                $other_ua,
                                option.emulation_os,
                            ),
                        )*
                        _ => $header_initializer(
                            $default_sec_ch_ua,
                            $default_ua,
                            EmulationOS::$default_os,
                        ),
                    };
                    Some(default_headers)
                } else {
                    None
                };

                $build_emulation(option, default_headers)
            }
        }
    };
}

mod_generator!(
    opera116,
    tls_config!(6, CURVES),
    http2_config!(3),
    header_initializer_with_zstd_priority,
    [
        (
            MacOS,
            r#""Opera";v="116", "Chromium";v="131", "Not_A Brand";v="24""#,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 OPR/116.0.0.0"
        ),
        (
            Windows,
            r#""Opera";v="116", "Chromium";v="131", "Not_A Brand";v="24""#,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 OPR/116.0.0.0"
        )
    ]
);

mod_generator!(
    opera117,
    opera116::build_emulation,
    header_initializer_with_zstd_priority,
    [
        (
            MacOS,
            r#""Not A(Brand";v="8", "Chromium";v="132", "Opera";v="117""#,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 OPR/117.0.0.0"
        ),
        (
            Windows,
            r#""Not A(Brand";v="8", "Chromium";v="132", "Opera";v="117""#,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 OPR/117.0.0.0"
        )
    ]
);

mod_generator!(
    opera118,
    opera116::build_emulation,
    header_initializer_with_zstd_priority,
    [
        (
            MacOS,
            r#""Not(A:Brand";v="99", "Opera";v="118", "Chromium";v="133""#,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 OPR/118.0.0.0"
        ),
        (
            Windows,
            r#""Not(A:Brand";v="99", "Opera";v="118", "Chromium";v="133""#,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 OPR/118.0.0.0"
        )
    ]
);

mod_generator!(
    opera119,
    opera116::build_emulation,
    header_initializer_with_zstd_priority,
    [
        (
            MacOS,
            r#""Chromium";v="134", "Not:A-Brand";v="24", "Opera";v="119""#,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 OPR/119.0.0.0"
        ),
        (
            Windows,
            r#""Chromium";v="134", "Not:A-Brand";v="24", "Opera";v="119""#,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 OPR/119.0.0.0"
        )
    ]
);
