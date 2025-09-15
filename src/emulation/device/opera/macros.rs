macro_rules! headers_stream_dependency {
    () => {
        StreamDependency::new(StreamId::zero(), 255, true)
    };
}

macro_rules! pseudo_order {
    () => {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Authority,
                PseudoId::Scheme,
                PseudoId::Path,
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

macro_rules! tls_options {
    ($curves:expr) => {
        OperaTlsConfig::builder()
            .curves($curves)
            .permute_extensions(true)
            .pre_shared_key(true)
            .enable_ech_grease(true)
            .build()
            .into()
    };
}

macro_rules! http2_options {
    () => {
        Http2Options::builder()
            .initial_window_size(6291456)
            .initial_connection_window_size(15728640)
            .max_header_list_size(262144)
            .header_table_size(65536)
            .enable_push(false)
            .headers_stream_dependency(headers_stream_dependency!())
            .headers_pseudo_order(pseudo_order!())
            .settings_order(settings_order!())
            .build()
    };
}

macro_rules! mod_generator {
    (
        $mod_name:ident,
        $tls_options:expr,
        $http2_options:expr,
        $header_initializer:ident,
        [($default_os:ident, $default_sec_ch_ua:tt, $default_ua:tt) $(, ($other_os:ident, $other_sec_ch_ua:tt, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            pub fn emulation(option: EmulationOption) -> Emulation {
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
        [($default_os:ident, $default_sec_ch_ua:tt, $default_ua:tt) $(, ($other_os:ident, $other_sec_ch_ua:tt, $other_ua:tt))*]
    ) => {
        pub(crate) mod $mod_name {
            use super::*;

            pub fn emulation(option: EmulationOption) -> Emulation {
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
