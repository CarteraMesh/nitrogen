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
pub struct MintAndWithdraw {
    pub mint_recipient: solana_pubkey::Pubkey,
    pub amount: u64,
    pub mint_token: solana_pubkey::Pubkey,
    pub fee_collected: u64,
}
