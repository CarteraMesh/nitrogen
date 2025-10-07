#[derive(
    Debug,
    borsh::BorshSerialize,
    bon::Builder,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Clone,
    Hash,
)]
pub struct LocalTokenRemoved {
    pub custody: solana_pubkey::Pubkey,
    pub mint: solana_pubkey::Pubkey,
}
