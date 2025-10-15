rm -rf /tmp/encoders
cargo run -p nitrogen-cli -- parse --idl ./idls/token_messenger_minter_v2.json --crate-name nitrogen-circle-token-messenger-minter-v2-encoder --output /tmp/encoders --filter deposit_for_burn,deposit_for_burn_with_hook &&
  cargo run -p nitrogen-cli -- parse --idl ./idls/message_transmitter_v2.json --crate-name nitrogen-circle-message-transmitter-v2-encoder --output /tmp/encoders --filter reclaim_event_account,receive_message
