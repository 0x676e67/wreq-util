#![allow(dead_code)]
pub mod delay_layer;
pub mod delay_server;
pub mod server;

use std::{
    cell::Cell,
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    num::Wrapping,
    sync::LazyLock,
    time::Duration,
};

use tokio::sync::Semaphore;
use wreq::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    http2::{
        Http2Options, PseudoId, PseudoOrder, SettingId, SettingsOrder, StreamDependency,
        StreamId,
    },
    tls::{CertificateCompressionAlgorithm, TlsOptions, TlsVersion},
    Client, Emulation,
};

// TODO: remove once done converting to new support server?
#[allow(unused)]
pub static DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub static TEST_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(60))
        .build()
        .unwrap()
});

// Fast random number generator (from wreq-util/src/emulation/rand.rs)
fn fast_random() -> u64 {
    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    #[inline]
    fn seed() -> u64 {
        let seed = RandomState::new();
        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}

/// Generates a random emulation configuration by randomly modifying TLS and HTTP/2 options.
///
/// This function creates a random `Emulation` by:
/// - Randomly selecting cipher suites, curves, and signature algorithms
/// - Randomly configuring TLS options (session ticket, grease, etc.)
/// - Randomly configuring HTTP/2 options (window sizes, stream dependencies, etc.)
/// - Randomly generating headers
///
/// The goal is to produce different JA4 fingerprints each time, without needing to match
/// real browser configurations.
///
/// # Example
///
/// ```rust,ignore
/// use crate::support::random_emulation_config;
///
/// let emulation = random_emulation_config();
/// let resp = crate::support::CLIENT
///     .get("https://example.com")
///     .emulation(emulation)
///     .send()
///     .await?;
/// ```
pub fn random_emulation_config() -> Emulation {
    let rand = fast_random();

    // Random cipher suites - ensure we always include TLS 1.3 ciphers for compatibility
    // Use predefined combinations to ensure valid configurations
    let cipher_lists = [
        "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        "TLS_AES_256_GCM_SHA384:TLS_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    ];
    let cipher_list = cipher_lists[((rand >> 24) as usize) % cipher_lists.len()];

    // Random curves - use predefined combinations for compatibility
    let curves_lists = [
        "X25519:P-256:P-384:P-521",
        "X25519:P-256:P-384",
        "X25519:P-256",
        "P-256:P-384:X25519",
    ];
    let curves_list = curves_lists[((rand >> 28) as usize) % curves_lists.len()];

    // Random signature algorithms - use a complete list for compatibility
    let sigalgs_lists = [
        "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384:rsa_pss_rsae_sha512:rsa_pkcs1_sha512",
        "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384",
        "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:ecdsa_secp384r1_sha384",
    ];
    let sigalgs_list = sigalgs_lists[((rand >> 48) as usize) % sigalgs_lists.len()];

    // Build TLS options with random configurations
    let mut tls_builder = TlsOptions::builder()
        .cipher_list(cipher_list)
        .curves_list(curves_list)
        .sigalgs_list(sigalgs_list)
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .grease_enabled((rand & 1) != 0)
        .session_ticket((rand & 2) != 0)
        .enable_ocsp_stapling((rand & 4) != 0)
        .enable_signed_cert_timestamps((rand & 8) != 0);

    // Randomly add certificate compression
    if (rand & 16) != 0 {
        tls_builder = tls_builder.certificate_compression_algorithms(&[
            CertificateCompressionAlgorithm::ZLIB,
        ]);
    }

    let tls_options = tls_builder.build();

    // Build HTTP/2 options with random configurations
    let initial_window_sizes = [2097152, 4194304, 6291456, 10485760];
    let connection_window_sizes = [10485760, 10551295, 15728640];
    let max_streams = [100, 1000];

    let mut http2_builder = Http2Options::builder()
        .initial_window_size(initial_window_sizes[((rand >> 8) as usize) % initial_window_sizes.len()])
        .initial_connection_window_size(
            connection_window_sizes[((rand >> 10) as usize) % connection_window_sizes.len()],
        )
        .max_concurrent_streams(max_streams[((rand >> 12) as usize) % max_streams.len()])
        .header_table_size(65536)
        .max_header_list_size(262144)
        .enable_push((rand & 32) == 0)
        .headers_stream_dependency(StreamDependency::new(
            StreamId::zero(),
            255,
            (rand & 64) != 0,
        ));

    // Random pseudo header order
    let pseudo_order = if (rand & 128) != 0 {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Scheme,
                PseudoId::Path,
                PseudoId::Authority,
            ])
            .build()
    } else {
        PseudoOrder::builder()
            .extend([
                PseudoId::Method,
                PseudoId::Authority,
                PseudoId::Scheme,
                PseudoId::Path,
            ])
            .build()
    };
    http2_builder = http2_builder.headers_pseudo_order(pseudo_order);

    // Random settings order
    let settings_order = if (rand & 256) != 0 {
        SettingsOrder::builder()
            .extend([
                SettingId::HeaderTableSize,
                SettingId::EnablePush,
                SettingId::InitialWindowSize,
                SettingId::MaxConcurrentStreams,
                SettingId::MaxFrameSize,
                SettingId::MaxHeaderListSize,
            ])
            .build()
    } else {
        SettingsOrder::builder()
            .extend([
                SettingId::HeaderTableSize,
                SettingId::MaxConcurrentStreams,
                SettingId::EnablePush,
                SettingId::InitialWindowSize,
                SettingId::MaxFrameSize,
                SettingId::MaxHeaderListSize,
            ])
            .build()
    };
    http2_builder = http2_builder.settings_order(settings_order);

    let http2_options = http2_builder.build();

    // Random headers
    let mut headers = HeaderMap::new();
    let user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
    ];
    let ua = user_agents[((rand >> 20) as usize) % user_agents.len()];
    headers.insert(USER_AGENT, HeaderValue::from_static(ua));

    // Build emulation
    Emulation::builder()
        .tls_options(tls_options)
        .http2_options(http2_options)
        .headers(headers)
        .build()
}

#[allow(unused_macros)]
macro_rules! test_emulation {
    ($test_name:ident, $emulation:expr, $ja4:expr, $akamai_hash:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let _permit = crate::support::TEST_SEMAPHORE.acquire().await.unwrap();

            let resp = crate::support::CLIENT
                .get("https://tls.browserleaks.com/")
                .emulation($emulation)
                .send()
                .await
                .unwrap();

            assert_eq!(resp.status(), wreq::StatusCode::OK);
            let content = resp.text().await.unwrap();

            let conditional = $ja4.iter().any(|&ja4| content.contains(ja4));
            if !conditional {
                println!("{}", content);
            }
            assert!(conditional);

            let conditional = content.contains($akamai_hash);
            if !conditional {
                println!("{}", content);
            }
            assert!(conditional);

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    };
}
