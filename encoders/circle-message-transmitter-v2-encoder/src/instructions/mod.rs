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

pub fn accept_ownership(
    params: crate::types::AcceptOwnershipParams,
) -> accept_ownership::AcceptOwnership {
    accept_ownership::AcceptOwnership { params }
}

pub fn disable_attester(
    params: crate::types::DisableAttesterParams,
) -> disable_attester::DisableAttester {
    disable_attester::DisableAttester { params }
}

pub fn enable_attester(
    params: crate::types::EnableAttesterParams,
) -> enable_attester::EnableAttester {
    enable_attester::EnableAttester { params }
}

pub fn initialize(params: crate::types::InitializeParams) -> initialize::Initialize {
    initialize::Initialize { params }
}
pub fn is_nonce_used() -> is_nonce_used::IsNonceUsed {
    is_nonce_used::IsNonceUsed {}
}

pub fn pause(params: crate::types::PauseParams) -> pause::Pause {
    pause::Pause { params }
}

pub fn receive_message(
    params: crate::types::ReceiveMessageParams,
) -> receive_message::ReceiveMessage {
    receive_message::ReceiveMessage { params }
}

pub fn reclaim_event_account(
    params: crate::types::ReclaimEventAccountParams,
) -> reclaim_event_account::ReclaimEventAccount {
    reclaim_event_account::ReclaimEventAccount { params }
}

pub fn send_message(params: crate::types::SendMessageParams) -> send_message::SendMessage {
    send_message::SendMessage { params }
}

pub fn set_max_message_body_size(
    params: crate::types::SetMaxMessageBodySizeParams,
) -> set_max_message_body_size::SetMaxMessageBodySize {
    set_max_message_body_size::SetMaxMessageBodySize { params }
}

pub fn set_signature_threshold(
    params: crate::types::SetSignatureThresholdParams,
) -> set_signature_threshold::SetSignatureThreshold {
    set_signature_threshold::SetSignatureThreshold { params }
}

pub fn transfer_ownership(
    params: crate::types::TransferOwnershipParams,
) -> transfer_ownership::TransferOwnership {
    transfer_ownership::TransferOwnership { params }
}

pub fn unpause(params: crate::types::UnpauseParams) -> unpause::Unpause {
    unpause::Unpause { params }
}

pub fn update_attester_manager(
    params: crate::types::UpdateAttesterManagerParams,
) -> update_attester_manager::UpdateAttesterManager {
    update_attester_manager::UpdateAttesterManager { params }
}

pub fn update_pauser(params: crate::types::UpdatePauserParams) -> update_pauser::UpdatePauser {
    update_pauser::UpdatePauser { params }
}
