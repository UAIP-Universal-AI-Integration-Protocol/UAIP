//! UAIP Core - Universal AI Integration Protocol Core Types
//!
//! This crate provides the fundamental types and message formats for the UAIP protocol.

pub mod device;
pub mod error;
pub mod message;
pub mod protocol;

pub use device::*;
pub use error::*;
pub use message::*;
pub use protocol::*;
