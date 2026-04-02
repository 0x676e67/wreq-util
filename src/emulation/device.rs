//! Emulation for different browsers.

#[macro_use]
mod macros;
pub mod chrome;
pub mod firefox;
pub mod okhttp;
pub mod opera;
pub mod safari;

pub use typed_builder::TypedBuilder;
#[cfg(feature = "emulation-compression")]
pub use wreq::header::ACCEPT_ENCODING;
pub use wreq::{
    Emulation, Group,
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

#[cfg(feature = "http3")]
pub use std::borrow::Cow;
#[cfg(feature = "http3")]
pub use wreq::http3::{
    Http3Options, PseudoId as H3PseudoId, PseudoOrder as H3PseudoOrder,
    SettingId as H3SettingId, TransportParameterConfig, TransportParameterId,
    TransportParameterKind, VersionEntry, VersionInformation,
};

pub use crate::emulation::{
    EmulationOS, EmulationOption,
    certificate::{BrotliCompressor, ZlibCompressor, ZstdCompressor},
};
