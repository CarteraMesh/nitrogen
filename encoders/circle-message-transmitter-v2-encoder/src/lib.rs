use solana_pubkey::declare_id;
pub struct MessageTransmitterV2Encoder;
pub mod accounts;
pub mod helpers;
pub mod instructions;
pub mod types;
pub use helpers::*;

declare_id!("CCTPV2Sm4AdWt5296sk4P66VBZ7bEhcARwFaaS9YPbeC");
