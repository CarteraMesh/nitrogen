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
pub struct UsedNonce {
    pub is_used: bool,
}
