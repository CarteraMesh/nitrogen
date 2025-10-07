#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct SetSignatureThresholdParams {
    pub new_signature_threshold: u32,
}
