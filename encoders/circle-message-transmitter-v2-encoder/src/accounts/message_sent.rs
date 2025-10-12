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
pub struct MessageSent {
    pub rent_payer: solana_pubkey::Pubkey,
    pub created_at: i64,
    pub message: Vec<u8>,
}
