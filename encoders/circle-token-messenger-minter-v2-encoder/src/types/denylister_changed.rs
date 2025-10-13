#[derive(
    Debug,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
    bon::Builder,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Clone,
    Hash,
)]
pub struct DenylisterChanged {
    pub old_denylister: solana_pubkey::Pubkey,
    pub new_denylister: solana_pubkey::Pubkey,
}
