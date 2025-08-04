macro_rules! headers_stream_dependency {
    (1) => {
        StreamDependency::new(StreamId::zero(), 255, true)
    };
    (2) => {
        StreamDependency::new(StreamId::zero(), 255, false)
    };
}

macro_rules! headers_pseudo_order {
    (1) => {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Scheme,
                PseudoId::Path,
                PseudoId::Authority,
            ])
            .build()
    };
    (2) => {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Scheme,
                PseudoId::Authority,
                PseudoId::Path,
            ])
            .build()
    };
}

macro_rules! settings_order {
    (1) => {
        SettingsOrder::builder()
            .extend([
                SettingId::HeaderTableSize,
                SettingId::EnablePush,
                SettingId::InitialWindowSize,
                SettingId::MaxConcurrentStreams,
                SettingId::MaxFrameSize,
                SettingId::MaxHeaderListSize,
                SettingId::EnableConnectProtocol,
                SettingId::NoRfc7540Priorities,
            ])
            .build()
    };
    (2) => {
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

macro_rules! tls_options {
    (1, $cipher_list:expr) => {
        SafariTlsConfig::builder()
            .cipher_list($cipher_list)
            .build()
            .into()
    };
    (2, $cipher_list:expr, $sigalgs_list:expr) => {
        SafariTlsConfig::builder()
            .cipher_list($cipher_list)
            .sigalgs_list($sigalgs_list)
            .build()
            .into()
    };
}

macro_rules! http2_options {
    (@base $builder:expr) => {
        $builder
            .max_concurrent_streams(100)
    };

    (1) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(2097152)
            .initial_connection_window_size(10551295)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .headers_pseudo_order(headers_pseudo_order!(1))
            .settings_order(settings_order!(1))
            .build()
    };
    (2) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(2097152)
            .initial_connection_window_size(10551295)
            .enable_push(false)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .headers_pseudo_order(headers_pseudo_order!(1))
            .settings_order(settings_order!(1))
            .build()
    };
    (3) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(2097152)
            .initial_connection_window_size(10485760)
            .enable_push(false)
            .enable_connect_protocol(true)
            .no_rfc7540_priorities(true)
            .headers_stream_dependency(headers_stream_dependency!(2))
            .headers_pseudo_order(headers_pseudo_order!(2))
            .settings_order(settings_order!(2))
            .build()
    };
    (4) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(4194304)
            .initial_connection_window_size(10551295)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .headers_pseudo_order(headers_pseudo_order!(1))
            .settings_order(settings_order!(1))
            .build()
    };
    (5) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(4194304)
            .initial_connection_window_size(10551295)
            .enable_push(false)
            .headers_stream_dependency(headers_stream_dependency!(1))
            .headers_pseudo_order(headers_pseudo_order!(1))
            .settings_order(settings_order!(1))
            .build()
    };
    (6) => {
        http2_options!(@base Http2Options::builder())
            .initial_window_size(2097152)
            .initial_connection_window_size(10485760)
            .enable_push(false)
            .no_rfc7540_priorities(true)
            .headers_stream_dependency(headers_stream_dependency!(2))
            .headers_pseudo_order(headers_pseudo_order!(2))
            .settings_order(settings_order!(2))
            .build()
    };
}

macro_rules! mod_generator {
    ($mod_name:ident, $tls_options:expr, $http2_options:expr, $header_initializer:ident, $ua:expr) => {
        pub(crate) mod $mod_name {
            use super::*;

            #[inline(always)]
            pub fn emulation(option: EmulationOption) -> Emulation {
                let default_headers = if !option.skip_headers {
                    Some($header_initializer($ua))
                } else {
                    None
                };

                build_emulation(option, default_headers)
            }

            #[inline(always)]
            pub fn build_emulation(
                option: EmulationOption,
                default_headers: Option<HeaderMap>,
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

    ($mod_name:ident, $build_emulation:expr, $header_initializer:ident, $ua:expr) => {
        pub(crate) mod $mod_name {
            use super::*;

            #[inline(always)]
            pub fn emulation(option: EmulationOption) -> Emulation {
                let default_headers = if !option.skip_headers {
                    Some($header_initializer($ua))
                } else {
                    None
                };

                $build_emulation(option, default_headers)
            }
        }
    };
}
