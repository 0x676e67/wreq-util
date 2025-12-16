//! Middleware helpers for `wreq`.
//!
//! Modules here are feature-gated. Enable the feature to make the middleware
//! available and to include its docs.

#[cfg(feature = "middleware-delay")]
pub mod delay;
