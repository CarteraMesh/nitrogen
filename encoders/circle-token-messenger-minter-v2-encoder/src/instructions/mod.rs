pub mod accept_ownership;
pub mod add_local_token;
pub mod add_remote_token_messenger;
pub mod burn_token_custody;
pub mod denylist_account;
pub mod deposit_for_burn;
pub mod deposit_for_burn_with_hook;
pub mod handle_receive_finalized_message;
pub mod handle_receive_unfinalized_message;
pub mod initialize;
pub mod link_token_pair;
pub mod pause;
pub mod remove_local_token;
pub mod remove_remote_token_messenger;
pub mod set_fee_recipient;
pub mod set_max_burn_amount_per_message;
pub mod set_min_fee;
pub mod set_min_fee_controller;
pub mod set_token_controller;
pub mod transfer_ownership;
pub mod undenylist_account;
pub mod unlink_token_pair;
pub mod unpause;
pub mod update_denylister;
pub mod update_pauser;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum TokenMessengerMinterV2Instruction {
    AcceptOwnership(accept_ownership::AcceptOwnership),
    AddLocalToken(add_local_token::AddLocalToken),
    AddRemoteTokenMessenger(add_remote_token_messenger::AddRemoteTokenMessenger),
    BurnTokenCustody(burn_token_custody::BurnTokenCustody),
    DenylistAccount(denylist_account::DenylistAccount),
    DepositForBurn(deposit_for_burn::DepositForBurn),
    DepositForBurnWithHook(deposit_for_burn_with_hook::DepositForBurnWithHook),
    HandleReceiveFinalizedMessage(handle_receive_finalized_message::HandleReceiveFinalizedMessage),
    HandleReceiveUnfinalizedMessage(handle_receive_unfinalized_message::HandleReceiveUnfinalizedMessage),
    Initialize(initialize::Initialize),
    LinkTokenPair(link_token_pair::LinkTokenPair),
    Pause(pause::Pause),
    RemoveLocalToken(remove_local_token::RemoveLocalToken),
    RemoveRemoteTokenMessenger(remove_remote_token_messenger::RemoveRemoteTokenMessenger),
    SetFeeRecipient(set_fee_recipient::SetFeeRecipient),
    SetMaxBurnAmountPerMessage(set_max_burn_amount_per_message::SetMaxBurnAmountPerMessage),
    SetMinFee(set_min_fee::SetMinFee),
    SetMinFeeController(set_min_fee_controller::SetMinFeeController),
    SetTokenController(set_token_controller::SetTokenController),
    TransferOwnership(transfer_ownership::TransferOwnership),
    UndenylistAccount(undenylist_account::UndenylistAccount),
    UnlinkTokenPair(unlink_token_pair::UnlinkTokenPair),
    Unpause(unpause::Unpause),
    UpdateDenylister(update_denylister::UpdateDenylister),
    UpdatePauser(update_pauser::UpdatePauser),
}

pub fn accept_ownership(params: crate::types::AcceptOwnershipParams) -> accept_ownership::AcceptOwnership {
    accept_ownership::AcceptOwnership {params }
}

pub fn add_local_token(params: crate::types::AddLocalTokenParams) -> add_local_token::AddLocalToken {
    add_local_token::AddLocalToken {params }
}

pub fn add_remote_token_messenger(params: crate::types::AddRemoteTokenMessengerParams) -> add_remote_token_messenger::AddRemoteTokenMessenger {
    add_remote_token_messenger::AddRemoteTokenMessenger {params }
}

pub fn burn_token_custody(params: crate::types::BurnTokenCustodyParams) -> burn_token_custody::BurnTokenCustody {
    burn_token_custody::BurnTokenCustody {params }
}

pub fn denylist_account(params: crate::types::DenylistParams) -> denylist_account::DenylistAccount {
    denylist_account::DenylistAccount {params }
}

pub fn deposit_for_burn(params: crate::types::DepositForBurnParams) -> deposit_for_burn::DepositForBurn {
    deposit_for_burn::DepositForBurn {params }
}

pub fn deposit_for_burn_with_hook(params: crate::types::DepositForBurnWithHookParams) -> deposit_for_burn_with_hook::DepositForBurnWithHook {
    deposit_for_burn_with_hook::DepositForBurnWithHook {params }
}

pub fn handle_receive_finalized_message(params: crate::types::HandleReceiveMessageParams) -> handle_receive_finalized_message::HandleReceiveFinalizedMessage {
    handle_receive_finalized_message::HandleReceiveFinalizedMessage {params }
}

pub fn handle_receive_unfinalized_message(params: crate::types::HandleReceiveMessageParams) -> handle_receive_unfinalized_message::HandleReceiveUnfinalizedMessage {
    handle_receive_unfinalized_message::HandleReceiveUnfinalizedMessage {params }
}

pub fn initialize(params: crate::types::InitializeParams) -> initialize::Initialize {
    initialize::Initialize {params }
}

pub fn link_token_pair(params: crate::types::LinkTokenPairParams) -> link_token_pair::LinkTokenPair {
    link_token_pair::LinkTokenPair {params }
}

pub fn pause(params: crate::types::PauseParams) -> pause::Pause {
    pause::Pause {params }
}

pub fn remove_local_token(params: crate::types::RemoveLocalTokenParams) -> remove_local_token::RemoveLocalToken {
    remove_local_token::RemoveLocalToken {params }
}

pub fn remove_remote_token_messenger(params: crate::types::RemoveRemoteTokenMessengerParams) -> remove_remote_token_messenger::RemoveRemoteTokenMessenger {
    remove_remote_token_messenger::RemoveRemoteTokenMessenger {params }
}

pub fn set_fee_recipient(params: crate::types::SetFeeRecipientParams) -> set_fee_recipient::SetFeeRecipient {
    set_fee_recipient::SetFeeRecipient {params }
}

pub fn set_max_burn_amount_per_message(params: crate::types::SetMaxBurnAmountPerMessageParams) -> set_max_burn_amount_per_message::SetMaxBurnAmountPerMessage {
    set_max_burn_amount_per_message::SetMaxBurnAmountPerMessage {params }
}

pub fn set_min_fee(params: crate::types::SetMinFeeParams) -> set_min_fee::SetMinFee {
    set_min_fee::SetMinFee {params }
}

pub fn set_min_fee_controller(params: crate::types::SetMinFeeControllerParams) -> set_min_fee_controller::SetMinFeeController {
    set_min_fee_controller::SetMinFeeController {params }
}

pub fn set_token_controller(params: crate::types::SetTokenControllerParams) -> set_token_controller::SetTokenController {
    set_token_controller::SetTokenController {params }
}

pub fn transfer_ownership(params: crate::types::TransferOwnershipParams) -> transfer_ownership::TransferOwnership {
    transfer_ownership::TransferOwnership {params }
}

pub fn undenylist_account(params: crate::types::UndenylistParams) -> undenylist_account::UndenylistAccount {
    undenylist_account::UndenylistAccount {params }
}

pub fn unlink_token_pair(params: crate::types::UninkTokenPairParams) -> unlink_token_pair::UnlinkTokenPair {
    unlink_token_pair::UnlinkTokenPair {params }
}

pub fn unpause(params: crate::types::UnpauseParams) -> unpause::Unpause {
    unpause::Unpause {params }
}

pub fn update_denylister(params: crate::types::UpdateDenylisterParams) -> update_denylister::UpdateDenylister {
    update_denylister::UpdateDenylister {params }
}

pub fn update_pauser(params: crate::types::UpdatePauserParams) -> update_pauser::UpdatePauser {
    update_pauser::UpdatePauser {params }
}