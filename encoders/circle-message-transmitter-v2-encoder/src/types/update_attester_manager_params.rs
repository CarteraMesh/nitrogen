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
pub struct UpdateAttesterManagerParams {
    pub new_attester_manager: solana_pubkey::Pubkey,
}
