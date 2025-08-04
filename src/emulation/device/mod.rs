//! Emulation for different browsers.
#![allow(missing_debug_implementations)]
#![allow(missing_docs)]

#[macro_use]
mod macros;
pub mod chrome;
pub mod firefox;
pub mod okhttp;
pub mod opera;
pub mod safari;

mod emulation_imports {
    #[cfg(all(feature = "gzip", feature = "deflate", feature = "brotli"))]
    pub use wreq::header::ACCEPT_ENCODING;
    pub use wreq::{
        Emulation,
        header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
        http2::Http2Options,
    };

    pub use crate::emulation::{EmulationOS, EmulationOption};
}

mod tls_imports {
    pub use typed_builder::TypedBuilder;
    pub use wreq::tls::{
        AlpnProtocol, AlpsProtocol, CertificateCompressionAlgorithm, ExtensionType, TlsOptions,
        TlsVersion,
    };
}

mod http2_imports {
    pub use wreq::http2::{
        Priorities, Priority, PseudoId, PseudoOrder, SettingId, SettingsOrder, StreamDependency,
        StreamId,
    };
}
