pub mod accept_ownership;
pub mod add_local_token;
pub mod add_remote_token_messenger;
pub mod burn_token_custody;
pub mod denylist_account;
pub mod denylisted_event;
pub mod denylister_changed_event;
pub mod deposit_for_burn;
pub mod deposit_for_burn_event;
pub mod deposit_for_burn_with_hook;
pub mod fee_recipient_set_event;
pub mod handle_receive_finalized_message;
pub mod handle_receive_unfinalized_message;
pub mod initialize;
pub mod link_token_pair;
pub mod local_token_added_event;
pub mod local_token_removed_event;
pub mod min_fee_controller_set_event;
pub mod min_fee_set_event;
pub mod mint_and_withdraw_event;
pub mod ownership_transfer_started_event;
pub mod ownership_transferred_event;
pub mod pause;
pub mod pause_event;
pub mod pauser_changed_event;
pub mod remote_token_messenger_added_event;
pub mod remote_token_messenger_removed_event;
pub mod remove_local_token;
pub mod remove_remote_token_messenger;
pub mod set_burn_limit_per_message_event;
pub mod set_fee_recipient;
pub mod set_max_burn_amount_per_message;
pub mod set_min_fee;
pub mod set_min_fee_controller;
pub mod set_token_controller;
pub mod set_token_controller_event;
pub mod token_custody_burned_event;
pub mod token_pair_linked_event;
pub mod token_pair_unlinked_event;
pub mod transfer_ownership;
pub mod un_denylisted_event;
pub mod undenylist_account;
pub mod unlink_token_pair;
pub mod unpause;
pub mod unpause_event;
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
    DenylistedEvent(denylisted_event::DenylistedEvent),
    DenylisterChangedEvent(denylister_changed_event::DenylisterChangedEvent),
    DepositForBurnEvent(deposit_for_burn_event::DepositForBurnEvent),
    FeeRecipientSetEvent(fee_recipient_set_event::FeeRecipientSetEvent),
    LocalTokenAddedEvent(local_token_added_event::LocalTokenAddedEvent),
    LocalTokenRemovedEvent(local_token_removed_event::LocalTokenRemovedEvent),
    MinFeeControllerSetEvent(min_fee_controller_set_event::MinFeeControllerSetEvent),
    MinFeeSetEvent(min_fee_set_event::MinFeeSetEvent),
    MintAndWithdrawEvent(mint_and_withdraw_event::MintAndWithdrawEvent),
    OwnershipTransferStartedEvent(ownership_transfer_started_event::OwnershipTransferStartedEvent),
    OwnershipTransferredEvent(ownership_transferred_event::OwnershipTransferredEvent),
    PauseEvent(pause_event::PauseEvent),
    PauserChangedEvent(pauser_changed_event::PauserChangedEvent),
    RemoteTokenMessengerAddedEvent(
        remote_token_messenger_added_event::RemoteTokenMessengerAddedEvent,
    ),
    RemoteTokenMessengerRemovedEvent(
        remote_token_messenger_removed_event::RemoteTokenMessengerRemovedEvent,
    ),
    SetBurnLimitPerMessageEvent(set_burn_limit_per_message_event::SetBurnLimitPerMessageEvent),
    SetTokenControllerEvent(set_token_controller_event::SetTokenControllerEvent),
    TokenCustodyBurnedEvent(token_custody_burned_event::TokenCustodyBurnedEvent),
    TokenPairLinkedEvent(token_pair_linked_event::TokenPairLinkedEvent),
    TokenPairUnlinkedEvent(token_pair_unlinked_event::TokenPairUnlinkedEvent),
    UnDenylistedEvent(un_denylisted_event::UnDenylistedEvent),
    UnpauseEvent(unpause_event::UnpauseEvent),
}
