//! Pure domain logic for Server Manager. No I/O, no async, no framework types.
pub mod model;
pub use model::*;
pub mod config;
pub use config::*;
pub mod migrate;
pub use migrate::*;
pub mod backoff;
pub use backoff::*;
pub mod wol;
pub use wol::*;
pub mod stats;
pub use stats::*;
