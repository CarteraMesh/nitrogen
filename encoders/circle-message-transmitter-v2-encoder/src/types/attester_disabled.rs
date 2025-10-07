#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct AttesterDisabled {
    pub attester: solana_pubkey::Pubkey,
}
