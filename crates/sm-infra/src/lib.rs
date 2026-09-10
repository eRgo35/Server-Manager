//! Infrastructure implementations for service traits.

pub mod probe;
pub mod wol;
pub use wol::UdpWaker;
