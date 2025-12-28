#[macro_use]
mod http2;
#[macro_use]
mod tls;
mod header;

use header::*;
use tls::*;

use super::*;

macro_rules! mod_generator {
    (
        $mod_name:ident,
        $tls_options:expr,
        $http2_options:expr,
        $header_initializer:ident,
        [($default_os:ident, $default_ua:tt) $(, ($other_os:ident, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            pub fn emulation(option: EmulationOption) -> Emulation {
                let default_headers = if !option.skip_headers {
                    #[allow(unreachable_patterns)]
                    let default_headers = match option.emulation_os {
                        $(
                            EmulationOS::$other_os => {
                                $header_initializer($other_ua)
                            }
                        ),*
                        _ => {
                            $header_initializer($default_ua)
                        }
                    };

                    Some(default_headers)
                } else {
                    None
                };

                build_emulation(option, default_headers)
            }

            pub fn build_emulation(
                option: EmulationOption,
                default_headers: Option<HeaderMap>
            ) -> Emulation {
                let mut builder = Emulation::builder().tls_options($tls_options);

                if !option.skip_http2 {
                    builder = builder.http2_options($http2_options);
                }

                if let Some(headers) = default_headers {
                    builder = builder.headers(headers);
                }

                builder.build()
            }
        }
    };
    (
        $mod_name:ident,
        $build_emulation:expr,
        $header_initializer:ident,
        [($default_os:ident, $default_ua:tt) $(, ($other_os:ident, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            pub fn emulation(option: EmulationOption) -> Emulation {
                let default_headers = if !option.skip_headers {
                    #[allow(unreachable_patterns)]
                    let default_headers = match option.emulation_os {
                        $(
                            EmulationOS::$other_os => {
                                $header_initializer($other_ua)
                            }
                        ),*
                        _ => {
                            $header_initializer($default_ua)
                        }
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
    ff109,
    tls_options!(2, CIPHER_LIST_1, CURVES_1),
    http2_options!(2),
    header_initializer,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/109.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_17; rv:109.0) Gecko/20000101 Firefox/109.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:109.0) Gecko/109.0 Firefox/109.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Linux i686; rv:109.0) Gecko/20100101 Firefox/109.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPad; CPU OS 13_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/109.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff117,
    ff109::build_emulation,
    header_initializer,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:117.0) Gecko/20100101 Firefox/117.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_16_1; rv:117.0) Gecko/20010101 Firefox/117.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:117.0) Gecko/117.0 Firefox/117.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Linux i686; rv:117.0) Gecko/20100101 Firefox/117.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/117.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff128,
    tls_options!(3, CIPHER_LIST_2, CURVES_1),
    http2_options!(3),
    header_initializer_with_zstd,
    [
        (
            MacOs,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:128.0) Gecko/20100101 Firefox/128.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:128.0) Gecko/128.0 Firefox/128.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/128.0 Mobile/15E148 Safari/605.1.15"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"
        )
    ]
);

mod_generator!(
    ff133,
    tls_options!(1, CIPHER_LIST_1, CURVES_2),
    http2_options!(1),
    header_initializer_with_zstd,
    [
        (
            MacOs,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:133.0) Gecko/20100101 Firefox/133.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/133.4 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff135,
    tls_options!(4, CIPHER_LIST_1, CURVES_2),
    http2_options!(1),
    header_initializer_with_zstd,
    [
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:135.0) Gecko/20100101 Firefox/135.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0"
        )
    ]
);

mod_generator!(
    ff_private_135,
    tls_options!(6, CIPHER_LIST_1, CURVES_2),
    http2_options!(1),
    header_initializer_with_zstd,
    [
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:135.0) Gecko/20100101 Firefox/135.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0"
        )
    ]
);

mod_generator!(
    ff_android_135,
    tls_options!(5, CIPHER_LIST_1, CURVES_1),
    http2_options!(4),
    header_initializer_with_zstd,
    [(
        Android,
        "Mozilla/5.0 (Android 13; Mobile; rv:135.0) Gecko/135.0 Firefox/135.0"
    )]
);

mod_generator!(
    ff136,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:136.0) Gecko/20100101 Firefox/136.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:136.0) Gecko/20100101 Firefox/136.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/136.0"
        )
    ]
);

mod_generator!(
    ff_private_136,
    ff_private_135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:136.0) Gecko/20100101 Firefox/136.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:136.0) Gecko/20100101 Firefox/136.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/136.0"
        )
    ]
);

mod_generator!(
    ff139,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:139.0) Gecko/20100101 Firefox/139.0"
        ),
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; rv:136.0) Gecko/20100101 Firefox/139.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:136.0) Gecko/20100101 Firefox/139.0"
        )
    ]
);

mod_generator!(
    ff142,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:142.0) Gecko/20100101 Firefox/142.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:142.0) Gecko/20100101 Firefox/142.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:142.0) Gecko/20100101 Firefox/142.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:142.0) Gecko/142.0 Firefox/142.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/142.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff143,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:143.0) Gecko/20100101 Firefox/143.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:143.0) Gecko/143.0 Firefox/143.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/143.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff144,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:144.0) Gecko/20100101 Firefox/144.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:144.0) Gecko/144.0 Firefox/144.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/144.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff145,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:145.0) Gecko/20100101 Firefox/145.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:145.0) Gecko/20100101 Firefox/145.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:145.0) Gecko/145.0 Firefox/145.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/145.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);

mod_generator!(
    ff146,
    ff135::build_emulation,
    header_initializer_with_zstd,
    [
        (
            Windows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:146.0) Gecko/20100101 Firefox/146.0"
        ),
        (
            MacOS,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:146.0) Gecko/20100101 Firefox/146.0"
        ),
        (
            Linux,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0"
        ),
        (
            Android,
            "Mozilla/5.0 (Android 13; Mobile; rv:146.0) Gecko/146.0 Firefox/146.0"
        ),
        (
            IOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/146.0 Mobile/15E148 Safari/605.1.15"
        )
    ]
);
