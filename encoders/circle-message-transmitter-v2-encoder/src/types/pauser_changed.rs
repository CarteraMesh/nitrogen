#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct PauserChanged {
    pub new_address: solana_pubkey::Pubkey,
}
