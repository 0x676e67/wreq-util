//! Client Hints middleware.
//!
//! Sends the client hints an origin asks for in `Accept-CH`, and restarts a request once when
//! `Critical-CH` names hints it lacked — the way Chrome does.
//!
//! # Quick Start
//!
//! Hints of a Chrome emulation:
//!
//! ```no_run
//! use wreq::Client;
//! use wreq_util::{
//!     Emulation,
//!     tower::client_hints::{ClientHintsLayer, Hints},
//! };
//!
//! let hints = Hints::from_emulation(Emulation::Chrome149);
//! let client = Client::builder()
//!     .emulation(Emulation::Chrome149)
//!     .layer(ClientHintsLayer::new(hints))
//!     .cookie_store(true)
//!     .build()?;
//! # Ok::<(), wreq::Error>(())
//! ```
//!
//! [`Hints::from_emulation`] covers the Google Chrome profiles and returns `None` for others.
//! Build [`Hints`] by hand for another Chromium browser or device.
//!
//! # Behavior
//!
//! - `Accept-CH` replaces the hints its origin accepts, and an empty one clears them
//! - `Critical-CH` naming an accepted hint the request lacked restarts it once with every
//!   accepted hint
//! - Only `GET` and `HEAD` are restarted — Chrome doesn't repeat a `POST` either
//! - Hints only go to `https` origins, and to `http` ones on the local machine
//! - A header the request sets itself is never replaced
//! - A request with its own `Sec-CH-UA` gets hints derived from its own headers
//! - Hint headers go where Chrome puts them, unless `orig_headers` sets the order
//!
//! # Notes
//!
//! - Hints follow the original request across redirects, where Chrome picks them per hop —
//!   disable redirects and follow them yourself to match
//! - A request with a streaming body is never restarted
//! - Enable the cookie store — `Critical-CH` servers usually expect the refused response's
//!   cookie back
//! - Keep high-entropy hints out of the client's default headers — the layer can't see them

mod future;
mod hints;
mod layer;
mod order;
mod parse;
mod service;
mod store;
mod versions;

pub use self::{
    future::ResponseFuture, hints::Hints, layer::ClientHintsLayer, service::ClientHints,
};
