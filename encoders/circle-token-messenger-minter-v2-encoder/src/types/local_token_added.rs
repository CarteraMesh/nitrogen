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
pub struct LocalTokenAdded {
    pub custody: solana_pubkey::Pubkey,
    pub mint: solana_pubkey::Pubkey,
}
