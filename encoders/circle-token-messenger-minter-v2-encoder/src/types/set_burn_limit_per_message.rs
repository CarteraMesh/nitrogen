#[derive(
    Debug,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
    bon::Builder,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Clone,
    Hash,
)]
pub struct SetBurnLimitPerMessage {
    pub token: solana_pubkey::Pubkey,
    pub burn_limit_per_message: u64,
}
