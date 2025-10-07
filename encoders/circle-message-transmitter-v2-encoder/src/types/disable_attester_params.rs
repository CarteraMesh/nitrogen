#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct DisableAttesterParams {
    pub attester: solana_pubkey::Pubkey,
}
