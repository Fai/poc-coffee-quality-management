//! Middleware for the Coffee Quality Management Platform

pub mod auth;

pub use auth::{auth_middleware, AuthUser, CurrentUser};
