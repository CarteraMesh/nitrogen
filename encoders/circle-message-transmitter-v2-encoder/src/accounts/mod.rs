pub mod message_sent;
pub use message_sent::*;
pub mod message_transmitter;
pub use message_transmitter::*;
pub mod used_nonce;
pub use used_nonce::*;

pub enum MessageTransmitterV2Account {
    MessageSent(message_sent::MessageSent),
    MessageTransmitter(message_transmitter::MessageTransmitter),
    UsedNonce(used_nonce::UsedNonce),
}
