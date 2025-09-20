//! Emulation for different browsers.

#[macro_use]
mod macros;
pub mod chrome;
pub mod firefox;
pub mod okhttp;
pub mod opera;
pub mod safari;

pub use typed_builder::TypedBuilder;
#[cfg(all(feature = "gzip", feature = "deflate", feature = "brotli"))]
pub use wreq::header::ACCEPT_ENCODING;
pub use wreq::{
    Emulation,
    header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    http2::{
        Http2Options, Priorities, Priority, PseudoId, PseudoOrder, SettingId, SettingsOrder,
        StreamDependency, StreamId,
    },
    tls::{
        AlpnProtocol, AlpsProtocol, CertificateCompressionAlgorithm, ExtensionType, TlsOptions,
        TlsVersion,
    },
};

pub use crate::emulation::{EmulationOS, EmulationOption};
