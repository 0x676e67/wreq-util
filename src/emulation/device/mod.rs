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
    pub use crate::emulation::{EmulationOS, EmulationOption};
    #[cfg(all(feature = "gzip", feature = "deflate", feature = "brotli"))]
    pub use wreq::header::ACCEPT_ENCODING;
    pub use wreq::header::{
        ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
    };
    pub use wreq::{EmulationProvider, Http2Config};
}

mod tls_imports {
    pub use typed_builder::TypedBuilder;
    pub use wreq::{
        AlpnProtos, AlpsProtos, CertCompressionAlgorithm, ExtensionType, SslCurve, TlsConfig,
        TlsVersion,
    };
}

mod http2_imports {
    pub use std::sync::LazyLock;
    pub use wreq::PseudoOrder::{self, *};
    pub use wreq::SettingsOrder::{self, *};
    pub use wreq::{Priority, StreamDependency, StreamId};
}
