use super::MessageTransmitterV2Encoder;
pub mod message_sent;
pub mod message_transmitter;
pub mod used_nonce;

pub enum MessageTransmitterV2Account {
    MessageSent(message_sent::MessageSent),
    MessageTransmitter(message_transmitter::MessageTransmitter),
    UsedNonce(used_nonce::UsedNonce),
}
