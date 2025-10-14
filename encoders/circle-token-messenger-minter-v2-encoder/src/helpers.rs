use {
    crate::{instructions::deposit_for_burn, types::DepositForBurnParams},
    nitrogen_instruction_builder::InstructionBuilder,
    solana_pubkey::{Pubkey, pubkey},
};

pub const SOLANA_DEV_USDC_ADDRESS: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
pub const SOLANA_MAIN_USDC_ADDRESS: Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub const MESSENGER_PROGRAM_ID: Pubkey = pubkey!("CCTPV2Sm4AdWt5296sk4P66VBZ7bEhcARwFaaS9YPbeC");
/// https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/examples/v2/utilsV2.ts
pub fn deposit_for_burn_instruction(
    params: DepositForBurnParams,
    owner: Pubkey,
    message_sent_event_account: Pubkey,
    circle_usdc_address: Pubkey,
) -> InstructionBuilder<crate::instructions::deposit_for_burn::DepositForBurn> {
    let remote_domain_id = params.destination_domain;
    let burn_token_account =
        spl_associated_token_account::get_associated_token_address(&owner, &circle_usdc_address);
    deposit_for_burn(params).accounts(
        owner,
        owner,
        burn_token_account,
        Pubkey::find_program_address(&[b"message_transmitter"], &MESSENGER_PROGRAM_ID).0,
        Pubkey::find_program_address(&[b"token_messenger"], &crate::ID).0,
        Pubkey::find_program_address(
            &[
                b"remote_token_messenger",
                remote_domain_id.to_string().as_bytes(),
            ],
            &crate::ID,
        )
        .0,
        Pubkey::find_program_address(&[b"token_minter"], &crate::ID).0,
        circle_usdc_address,
        message_sent_event_account,
        crate::ID,
    )
}
