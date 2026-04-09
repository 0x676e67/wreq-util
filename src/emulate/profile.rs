//! Emulation for different browsers.

#[macro_use]
mod macros;
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
    Emulation, Platform,
    compress::{BrotliCompressor, ZlibCompressor, ZstdCompressor},
};
