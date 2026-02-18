//! Async-first Rust bindings for the KiCad IPC API.
//!
//! This crate is intentionally layered:
//! - transport
//! - envelope
//! - command builders
//! - high-level client

pub mod client;
pub mod envelope;
pub mod error;
pub mod model;
pub mod transport;

pub mod commands;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use crate::client::{ClientBuilder, KiCadClient};
pub use crate::error::KiCadError;
