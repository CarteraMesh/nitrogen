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
pub struct SetFeeRecipientParams {
    pub new_fee_recipient: solana_pubkey::Pubkey,
}
