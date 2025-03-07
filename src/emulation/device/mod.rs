//! Emulation for different browsers.
#![allow(missing_debug_implementations)]
#![allow(missing_docs)]

#[macro_use]
mod macros;
pub mod chrome;
pub mod firefox;
pub mod okhttp;
pub mod safari;

mod emulation_imports {
    pub use crate::emulation::{EmulationOS, EmulationOption};
    #[cfg(all(feature = "gzip", feature = "deflate", feature = "brotli"))]
    pub use rquest::header::ACCEPT_ENCODING;
    pub use rquest::header::{
        ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, UPGRADE_INSECURE_REQUESTS,
        USER_AGENT,
    };
    pub use rquest::{EmulationProvider, Http2Config};
}

mod tls_imports {
    pub use rquest::{
        AlpnProtos, AlpsProtos, CertCompressionAlgorithm, ExtensionType, SslCurve, TlsConfig,
        TlsVersion,
    };
    pub use typed_builder::TypedBuilder;
}

mod http2_imports {
    pub use rquest::PseudoOrder::{self, *};
    pub use rquest::SettingsOrder::{self, *};
    pub use rquest::{Priority, StreamDependency, StreamId};
    pub use std::sync::LazyLock;
}
