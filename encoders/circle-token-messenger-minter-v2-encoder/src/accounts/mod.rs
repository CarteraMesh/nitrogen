pub mod denylisted_account;
pub mod local_token;
pub mod message_transmitter;
pub mod remote_token_messenger;
pub mod token_messenger;
pub mod token_minter;
pub mod token_pair;

pub enum TokenMessengerMinterV2Account {
    DenylistedAccount(denylisted_account::DenylistedAccount),
    LocalToken(local_token::LocalToken),
    MessageTransmitter(message_transmitter::MessageTransmitter),
    RemoteTokenMessenger(remote_token_messenger::RemoteTokenMessenger),
    TokenMessenger(token_messenger::TokenMessenger),
    TokenMinter(token_minter::TokenMinter),
    TokenPair(token_pair::TokenPair),
}
