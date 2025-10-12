#[derive(
    Debug,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
    bon::Builder,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Default,
    Eq,
    Clone,
    Hash,
)]
pub struct RemoteTokenMessenger {
    pub domain: u32,
    pub token_messenger: solana_pubkey::Pubkey,
}
