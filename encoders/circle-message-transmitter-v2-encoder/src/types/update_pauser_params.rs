#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct UpdatePauserParams {
    pub new_pauser: solana_pubkey::Pubkey,
}
