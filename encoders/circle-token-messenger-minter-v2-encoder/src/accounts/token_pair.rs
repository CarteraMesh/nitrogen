#[derive(
    Debug,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
    bon::Builder,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Default,
    Eq,
    Clone,
    Hash,
)]
pub struct TokenPair {
    pub remote_domain: u32,
    pub remote_token: solana_pubkey::Pubkey,
    pub local_token: solana_pubkey::Pubkey,
    pub bump: u8,
}
