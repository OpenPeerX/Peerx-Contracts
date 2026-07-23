#![deny(missing_docs)]
//! Market Data Streaming
//! 
//! A high-performance real-time market data streaming system for Soroban.
//! 
//! Provides WebSocket feeds, order book management, data compression,
//! and historical data access.
pub mod websocket;
pub mod orderbook;
pub mod compression;
pub mod connection;
pub mod validation;
pub mod rate_limiter;
pub mod historical;
pub mod data_sources;
pub mod types;
pub mod error;

pub use websocket::*;
pub use orderbook::*;
pub use compression::*;
pub use connection::*;
pub use validation::*;
pub use rate_limiter::*;
pub use historical::*;
pub use data_sources::*;
pub use types::*;
pub use error::*;
