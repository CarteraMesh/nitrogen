pub mod accept_ownership;
pub mod disable_attester;
pub mod enable_attester;
pub mod initialize;
pub mod is_nonce_used;
pub mod pause;
pub mod receive_message;
pub mod reclaim_event_account;
pub mod send_message;
pub mod set_max_message_body_size;
pub mod set_signature_threshold;
pub mod transfer_ownership;
pub mod unpause;
pub mod update_attester_manager;
pub mod update_pauser;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum MessageTransmitterV2Instruction {
    AcceptOwnership(accept_ownership::AcceptOwnership),
    DisableAttester(disable_attester::DisableAttester),
    EnableAttester(enable_attester::EnableAttester),
    Initialize(initialize::Initialize),
    IsNonceUsed(is_nonce_used::IsNonceUsed),
    Pause(pause::Pause),
    ReceiveMessage(receive_message::ReceiveMessage),
    ReclaimEventAccount(reclaim_event_account::ReclaimEventAccount),
    SendMessage(send_message::SendMessage),
    SetMaxMessageBodySize(set_max_message_body_size::SetMaxMessageBodySize),
    SetSignatureThreshold(set_signature_threshold::SetSignatureThreshold),
    TransferOwnership(transfer_ownership::TransferOwnership),
    Unpause(unpause::Unpause),
    UpdateAttesterManager(update_attester_manager::UpdateAttesterManager),
    UpdatePauser(update_pauser::UpdatePauser),
}
