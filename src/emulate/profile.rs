//! Emulation for different browsers.

pub mod chrome;
pub mod firefox;
pub mod okhttp;
pub mod opera;
pub mod safari;

use typed_builder::TypedBuilder;
#[cfg(feature = "emulation-compression")]
use wreq::header::ACCEPT_ENCODING;
use wreq::{
    Group,
    header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    http2::{
        Http2Options, Priorities, Priority, PseudoId, PseudoOrder, SettingId, SettingsOrder,
        StreamDependency, StreamId,
    },
    tls::{
        AlpnProtocol, AlpsProtocol, ExtensionType, KeyShare, TlsOptions, TlsVersion,
        compress::CertificateCompressor,
    },
};

use super::{
    Emulation, Platform, Strategy,
    compress::{BrotliCompressor, ZlibCompressor, ZstdCompressor},
};

const SEC_CH_UA: HeaderName = HeaderName::from_static("sec-ch-ua");
const SEC_CH_UA_MOBILE: HeaderName = HeaderName::from_static("sec-ch-ua-mobile");
const SEC_CH_UA_PLATFORM: HeaderName = HeaderName::from_static("sec-ch-ua-platform");
const UPGRADE_INSECURE_REQUESTS: HeaderName = HeaderName::from_static("upgrade-insecure-requests");
const SEC_FETCH_SITE: HeaderName = HeaderName::from_static("sec-fetch-site");
const SEC_FETCH_MODE: HeaderName = HeaderName::from_static("sec-fetch-mode");
const SEC_FETCH_USER: HeaderName = HeaderName::from_static("sec-fetch-user");
const SEC_FETCH_DEST: HeaderName = HeaderName::from_static("sec-fetch-dest");
const PRIORITY: HeaderName = HeaderName::from_static("priority");
const TE: HeaderName = HeaderName::from_static("te");

fn build_standard_emulation(
    group: &'static str,
    tls_options: TlsOptions,
    http2_options: Option<Http2Options>,
    default_headers: Option<HeaderMap>,
) -> wreq::Emulation {
    let mut builder = wreq::Emulation::builder().tls_options(tls_options);

    if let Some(http2_options) = http2_options {
        builder = builder.http2_options(http2_options);
    }

    if let Some(headers) = default_headers {
        builder = builder.headers(headers);
    }

    builder.build(Group::new(group))
}
