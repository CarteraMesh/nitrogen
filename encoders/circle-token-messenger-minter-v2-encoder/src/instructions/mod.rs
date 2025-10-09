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
    HandleReceiveUnfinalizedMessage(
        handle_receive_unfinalized_message::HandleReceiveUnfinalizedMessage,
    ),
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
