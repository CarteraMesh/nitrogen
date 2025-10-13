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
pub struct TokenPairLinked {
    pub local_token: solana_pubkey::Pubkey,
    pub remote_domain: u32,
    pub remote_token: solana_pubkey::Pubkey,
}
