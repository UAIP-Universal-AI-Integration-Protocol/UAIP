//! Protocol constants and version information

/// UAIP Protocol version
pub const PROTOCOL_VERSION: &str = "1.0";

/// Default port for UAIP Hub
pub const DEFAULT_PORT: u16 = 8443;

/// Message types
pub const MSG_TYPE_COMMAND: &str = "command";
pub const MSG_TYPE_TELEMETRY: &str = "telemetry";
pub const MSG_TYPE_EVENT: &str = "event";
pub const MSG_TYPE_STREAM: &str = "stream";
pub const MSG_TYPE_ERROR: &str = "error";

/// Priority levels
pub const PRIORITY_CRITICAL: u8 = 0;
pub const PRIORITY_HIGH: u8 = 1;
pub const PRIORITY_NORMAL: u8 = 2;
pub const PRIORITY_LOW: u8 = 3;

/// QoS levels
pub const QOS_FIRE_AND_FORGET: u8 = 0;
pub const QOS_AT_LEAST_ONCE: u8 = 1;
pub const QOS_EXACTLY_ONCE: u8 = 2;
