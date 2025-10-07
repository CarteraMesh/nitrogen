#[derive(
    Debug, borsh::BorshSerialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash,
)]
pub struct SetMaxMessageBodySizeParams {
    pub new_max_message_body_size: u64,
}
