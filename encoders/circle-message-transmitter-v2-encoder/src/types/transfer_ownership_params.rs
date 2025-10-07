#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct TransferOwnershipParams {
    pub new_owner: solana_pubkey::Pubkey,
}
