use {
    crate::{instructions::deposit_for_burn, types::DepositForBurnParams},
    nitrogen_instruction_builder::InstructionBuilder,
    solana_pubkey::Pubkey,
};

/// https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/examples/v2/utilsV2.ts
pub fn deposit_for_burn_instruction(
    params: DepositForBurnParams,
    owner: Pubkey,
    remote_domain_id: u32,
    message_sent_event_account: Pubkey,
    circle_usdc_address: Pubkey,
    token_minter_program_id: Pubkey,
) -> InstructionBuilder<crate::instructions::deposit_for_burn::DepositForBurn> {
    let burn_token_account =
        spl_associated_token_account::get_associated_token_address(&owner, &circle_usdc_address);
    deposit_for_burn(params).accounts(
        owner,
        owner,
        burn_token_account,
        Pubkey::find_program_address(&[b"message_transmitter"], &crate::ID).0,
        Pubkey::find_program_address(&[b"token_messenger"], &token_minter_program_id).0,
        Pubkey::find_program_address(
            &[
                b"remote_token_messenger",
                remote_domain_id.to_string().as_bytes(),
            ],
            &token_minter_program_id,
        )
        .0,
        Pubkey::find_program_address(&[b"token_minter"], &token_minter_program_id).0,
        circle_usdc_address,
        message_sent_event_account,
        crate::ID,
    )
}
