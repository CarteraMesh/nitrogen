#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct ReclaimEventAccountParams {
    pub attestation: Vec<u8>,
    pub destination_message: Vec<u8>,
}
