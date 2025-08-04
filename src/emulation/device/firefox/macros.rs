macro_rules! headers_stream_dependency {
    (1) => {
        StreamDependency::new(StreamId::zero(), 41, false)
    };
    (2) => {
        StreamDependency::new(StreamId::from(13), 41, false)
    };
}

macro_rules! pseudo_order {
    () => {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Path,
                PseudoId::Authority,
                PseudoId::Scheme,
            ])
            .build()
    };
}

macro_rules! settings_order {
    () => {
        SettingsOrder::builder()
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
            .build()
    };
}

macro_rules! http2_options {
    (@base $builder:expr) => {
        $builder
            .initial_window_size(131072)
            .max_frame_size(16384)
            .initial_connection_window_size(12517377 + 65535)
            .headers_pseudo_order(pseudo_order!())
            .settings_order(settings_order!())
    };

    (1) => {
        http2_options!(@base Http2Options::builder())
            .initial_stream_id(3)
            .header_table_size(65536)
            .enable_push(false)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .build()
    };
    (2) => {
        http2_options!(@base Http2Options::builder())
            .initial_stream_id(15)
            .header_table_size(65536)
            .headers_stream_dependency(headers_stream_dependency!(2))
            .priorities(
                Priorities::builder()
                    .extend([
                        Priority::new(
                            StreamId::from(3),
                            StreamDependency::new(StreamId::zero(), 200, false),
                        ),
                        Priority::new(
                            StreamId::from(5),
                            StreamDependency::new(StreamId::zero(), 100, false),
                        ),
                        Priority::new(
                            StreamId::from(7),
                            StreamDependency::new(StreamId::zero(), 0, false),
                        ),
                        Priority::new(
                            StreamId::from(9),
                            StreamDependency::new(StreamId::from(7), 0, false),
                        ),
                        Priority::new(
                            StreamId::from(11),
                            StreamDependency::new(StreamId::from(3), 0, false),
                        ),
                        Priority::new(
                            StreamId::from(13),
                            StreamDependency::new(StreamId::zero(), 240, false),
                        ),
                    ])
                    .build(),
            )
            .build()
    };
    (3) => {
        http2_options!(@base Http2Options::builder())
            .initial_stream_id(3)
            .header_table_size(65536)
            .enable_push(false)
            .max_concurrent_streams(0)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .build()
    };
    (4) => {
        http2_options!(@base Http2Options::builder())
            .initial_stream_id(3)
            .header_table_size(4096)
            .enable_push(false)
            .initial_window_size(32768)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .build()
    };
}

macro_rules! tls_options {
    (@build $builder:expr) => {
        $builder.build().into()
    };

    (@base $builder:expr, $cipher_list:expr, $curves:expr) => {
        $builder
            .cipher_list($cipher_list)
            .curves_list($curves)
    };

    (1, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .psk_skip_session_tickets(true)
            .key_shares_limit(3)
            .certificate_compression_algorithms(CERT_COMPRESSION_ALGORITHM))
    };
    (2, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .key_shares_limit(2))
    };
    (3, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .session_ticket(false)
            .enable_ech_grease(true)
            .psk_dhe_ke(false)
            .key_shares_limit(2))
    };
    (4, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .enable_ech_grease(true)
            .enable_signed_cert_timestamps(true)
            .session_ticket(true)
            .pre_shared_key(true)
            .psk_skip_session_tickets(true)
            .key_shares_limit(3)
            .certificate_compression_algorithms(CERT_COMPRESSION_ALGORITHM))
    };
    (5, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .enable_ech_grease(true)
            .pre_shared_key(true)
            .psk_skip_session_tickets(true)
            .key_shares_limit(2)
            .certificate_compression_algorithms(CERT_COMPRESSION_ALGORITHM))
    };
    (6, $cipher_list:expr, $curves:expr) => {
        tls_options!(@build tls_options!(@base FirefoxTlsConfig::builder(), $cipher_list, $curves)
            .enable_ech_grease(true)
            .enable_signed_cert_timestamps(true)
            .session_ticket(false)
            .psk_dhe_ke(false)
            .key_shares_limit(3)
            .certificate_compression_algorithms(CERT_COMPRESSION_ALGORITHM))
    };
}

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

            #[inline(always)]
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

            #[inline(always)]
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

            #[inline(always)]
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
