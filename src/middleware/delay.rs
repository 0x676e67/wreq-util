//! Tower middleware for introducing delays between requests.
//!
//! This module provides `DelayLayer` and `Delay`, a small Tower `Layer` and
//! its corresponding service that introduce an asynchronous delay between
//! requests. The layer is primarily useful for client-side request pacing —
//! to avoid bursts, simulate slower networks, or throttle request rates in
//! tests and examples.
//!
//! Example:
//!
//! ```no_run
//! use std::time::Duration;
//! use wreq::Client;
//! use wreq_util::tower::delay::DelayLayer;
//!
//! let client = Client::builder()
//!     .layer(DelayLayer::new(Duration::from_secs(1)))
//!     .build()?;
//! # Ok::<(), wreq::Error>(())
//! ```
//!
//! Behavior notes:
//! - The layer inserts the configured delay in an async-aware fashion before forwarding a request
//!   to the inner service. It uses non-blocking timers so it will not block the runtime thread.
//! - This is intended for pacing and simulation. It should not be relied on as a strict
//!   rate-limiting or security mechanism; remote servers can still observe timing characteristics.
//! - Avoid placing long delays in performance-critical synchronous code paths as that will reduce
//!   throughput. Prefer shorter, explicit pacing values appropriate for your use case.
//!
//! This module re-exports `DelayLayer` and the response future for
//! convenience.

mod future;
mod layer;

pub use self::{
    future::ResponseFuture,
    layer::{Delay, DelayLayer},
};
