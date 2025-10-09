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
pub struct PauserChanged {
    pub new_address: solana_pubkey::Pubkey,
}
