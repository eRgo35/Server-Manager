//! Infrastructure implementations for service traits.

pub mod known_hosts;
pub mod probe;
pub mod wol;
pub use wol::UdpWaker;
