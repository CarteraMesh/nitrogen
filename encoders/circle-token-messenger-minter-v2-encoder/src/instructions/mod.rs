pub mod deposit_for_burn;
pub mod deposit_for_burn_with_hook;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum TokenMessengerMinterV2Instruction {
    DepositForBurn(deposit_for_burn::DepositForBurn),
    DepositForBurnWithHook(deposit_for_burn_with_hook::DepositForBurnWithHook),
}

pub fn deposit_for_burn(
    params: crate::types::DepositForBurnParams,
) -> deposit_for_burn::DepositForBurn {
    deposit_for_burn::DepositForBurn { params }
}

pub fn deposit_for_burn_with_hook(
    params: crate::types::DepositForBurnWithHookParams,
) -> deposit_for_burn_with_hook::DepositForBurnWithHook {
    deposit_for_burn_with_hook::DepositForBurnWithHook { params }
}
