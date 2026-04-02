pub const QUIC_SIGALGS_LIST: &str = join!(
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

macro_rules! chrome_tp_config {
    () => {
        TransportParameterConfig::new(
            vec![
                TransportParameterKind::Known(TransportParameterId::MaxIdleTimeout),
                TransportParameterKind::Known(TransportParameterId::MaxUdpPayloadSize),
                TransportParameterKind::Known(TransportParameterId::InitialMaxData),
                TransportParameterKind::Known(TransportParameterId::InitialMaxStreamDataBidiLocal),
                TransportParameterKind::Known(TransportParameterId::InitialMaxStreamDataBidiRemote),
                TransportParameterKind::Known(TransportParameterId::InitialMaxStreamDataUni),
                TransportParameterKind::Known(TransportParameterId::InitialMaxStreamsBidi),
                TransportParameterKind::Known(TransportParameterId::InitialMaxStreamsUni),
                TransportParameterKind::Known(TransportParameterId::InitialSourceConnectionId),
                TransportParameterKind::VersionInformation(VersionInformation {
                    chosen_version: 1,
                    available: vec![VersionEntry::Real(1), VersionEntry::Grease],
                }),
                TransportParameterKind::Known(TransportParameterId::MaxDatagramFrameSize),
                // Chrome-specific "ORIG" marker (0x3128)
                TransportParameterKind::Custom {
                    id: 12584,
                    value: vec![0x4f, 0x52, 0x49, 0x47],
                },
                TransportParameterKind::Grease,
            ],
            true, // Fisher-Yates shuffle like Chrome
        )
    };
}

macro_rules! h3_pseudo_order {
    () => {
        H3PseudoOrder::builder()
            .push(H3PseudoId::Method)
            .push(H3PseudoId::Authority)
            .push(H3PseudoId::Scheme)
            .push(H3PseudoId::Path)
            .build()
    };
}

macro_rules! http3_options {
    (1, $curves:expr) => {
        Http3Options::builder()
            // HTTP/3 SETTINGS: 1:65536;6:262144;7:100;51:1;GREASE
            .qpack_max_table_capacity(65536u64)
            .max_field_section_size(262144u64)
            .qpack_blocked_streams(100u64)
            .enable_datagram(true)
            .send_grease(true)
            .settings_order(vec![
                H3SettingId::QPACK_MAX_TABLE_CAPACITY,
                H3SettingId::MAX_HEADER_LIST_SIZE,
                H3SettingId::QPACK_MAX_BLOCKED_STREAMS,
                H3SettingId::H3_DATAGRAM,
            ])
            .pseudo_header_order(h3_pseudo_order!())
            // QUIC transport parameters
            .transport_parameter_config(chrome_tp_config!())
            .max_idle_timeout(30_000u64)
            .conn_receive_window(15_728_640u32)
            .stream_receive_window(6_291_456u32)
            .max_concurrent_bidi_streams(100u32)
            .max_concurrent_uni_streams(103u32)
            .datagram_receive_buffer_size(65536usize)
            // Zero-length source CIDs like Chrome
            .connection_id_length(0usize)
            // 8-byte initial destination CIDs like Chrome
            .initial_dst_cid_length(8usize)
            // QUIC TLS overrides (different from TCP TLS)
            .quic_curves_list(Cow::Borrowed($curves))
            .quic_sigalgs_list(Cow::Borrowed(QUIC_SIGALGS_LIST))
            .quic_grease_enabled(false)
            .quic_enable_ocsp_stapling(false)
            .quic_enable_signed_cert_timestamps(false)
            .quic_certificate_compressors(Some(Cow::Borrowed(CERT_COMPRESSION_ALGORITHM)))
            .quic_alps_protocols(vec![b"h3".to_vec()])
            .quic_alps_use_new_codepoint(true)
            .quic_ech_grease(true)
            .build()
    };
}
