#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct AttesterManagerUpdated {
    pub previous_attester_manager: solana_pubkey::Pubkey,
    pub new_attester_manager: solana_pubkey::Pubkey,
}
