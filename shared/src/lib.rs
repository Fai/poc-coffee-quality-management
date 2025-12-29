//! Shared types and models for the Coffee Quality Management Platform
//!
//! This crate contains types shared between the backend, frontend (via WASM),
//! and other components of the system.

pub mod models;
pub mod types;
pub mod validation;

pub use models::*;
pub use types::*;
pub use validation::*;
