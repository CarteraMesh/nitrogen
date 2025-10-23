use {
    super::FeeRecipientFetcher,
    crate::{instructions::receive_message, types::ReceiveMessageParams},
    alloy_primitives::FixedBytes,
    borsh::BorshDeserialize,
    nitrogen_instruction_builder::{InstructionBuilder, derive_pda},
    solana_instruction::AccountMeta,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_rpc_client_api::client_error::Result as ClientResult,
};

#[async_trait::async_trait]
impl<T: AsRef<RpcClient> + Send + Sync> FeeRecipientFetcher for T {
    async fn get_fee_recipient_token_account(
        &self,
        circle_usdc_address: &Pubkey,
    ) -> ClientResult<Pubkey> {
        let token_messenger_account =
            Pubkey::find_program_address(&[b"token_messenger"], &super::TOKEN_MINTER_PROGRAM_ID).0;
        let account_data = self
            .as_ref()
            .get_account_data(&token_messenger_account)
            .await?;
        let fee_recipient = TokenMessenger::try_from_slice(&account_data[8..])?.fee_recipient;
        Ok(spl_associated_token_account::get_associated_token_address(
            &fee_recipient,
            circle_usdc_address,
        ))
    }
}

// TODO maybe depend on token_minter crate?
#[derive(Debug, borsh::BorshSerialize, borsh::BorshDeserialize)]
struct TokenMessenger {
    pub denylister: solana_pubkey::Pubkey,
    pub owner: solana_pubkey::Pubkey,
    pub pending_owner: solana_pubkey::Pubkey,
    pub message_body_version: u32,
    pub authority_bump: u8,
    pub fee_recipient: solana_pubkey::Pubkey,
    pub min_fee_controller: solana_pubkey::Pubkey,
    pub min_fee: u32,
}

pub async fn fee_recipient_token_account_internal(
    account_data: &[u8],
    circle_usdc_address: &Pubkey,
) -> ClientResult<Pubkey> {
    let fee_recipient = TokenMessenger::try_from_slice(&account_data[8..])?.fee_recipient;
    Ok(spl_associated_token_account::get_associated_token_address(
        &fee_recipient,
        circle_usdc_address,
    ))
}

pub async fn fee_recipient_token_account<T: FeeRecipientFetcher>(
    rpc: &T,
    circle_usdc_address: &Pubkey,
) -> ClientResult<Pubkey> {
    let fee_recipient = rpc
        .get_fee_recipient_token_account(circle_usdc_address)
        .await?;
    Ok(spl_associated_token_account::get_associated_token_address(
        &fee_recipient,
        circle_usdc_address,
    ))
}

pub fn recv_from_attestation(
    destination_owner: Pubkey,
    token_minter_program_id: Pubkey,
    attestation: Vec<u8>,
    message: Vec<u8>,
) -> InstructionBuilder<crate::instructions::receive_message::ReceiveMessage> {
    let used_nonce =
        Pubkey::find_program_address(&[b"used_nonce", &message[12..12 + 32]], &crate::ID).0;

    receive_message(
        ReceiveMessageParams::builder()
            .attestation(attestation)
            .message(message)
            .build(),
    )
    .accounts(
        destination_owner,
        destination_owner,
        Pubkey::find_program_address(&[b"message_transmitter"], &crate::ID).0,
        used_nonce,
        token_minter_program_id,
        crate::ID,
    )
}

/// https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/examples/v2/utilsV2.ts
pub fn remaining_accounts(
    recipient: &Pubkey,
    remote_domain: String,
    remote_address: FixedBytes<32>,
    token_minter_program: &Pubkey,
    circle_usdc_address: &Pubkey,
    fee_recipient_token_account: &Pubkey,
) -> Vec<AccountMeta> {
    let user_token_account =
        spl_associated_token_account::get_associated_token_address(recipient, circle_usdc_address);
    let remote_address = Pubkey::new_from_array(remote_address.into());
    vec![
        derive_pda(&[b"token_messenger"], token_minter_program, true),
        derive_pda(
            &[b"remote_token_messenger", remote_domain.as_bytes()],
            token_minter_program,
            true,
        ),
        derive_pda(&[b"token_minter"], token_minter_program, false),
        derive_pda(
            &[b"local_token", circle_usdc_address.as_ref()],
            token_minter_program,
            false,
        ),
        derive_pda(
            &[
                b"token_pair",
                remote_domain.as_bytes(),
                remote_address.as_ref(),
            ],
            token_minter_program,
            true,
        ),
        AccountMeta::new(*fee_recipient_token_account, false),
        AccountMeta::new(user_token_account, false),
        derive_pda(
            &[b"custody", circle_usdc_address.as_ref()],
            token_minter_program,
            false,
        ),
        AccountMeta::new_readonly(spl_token::ID, false),
        derive_pda(&[b"__event_authority"], token_minter_program, true),
        AccountMeta::new_readonly(*token_minter_program, false),
    ]
}
