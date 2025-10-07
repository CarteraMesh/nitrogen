#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct EnableAttesterParams {
    pub new_attester: solana_pubkey::Pubkey,
}
