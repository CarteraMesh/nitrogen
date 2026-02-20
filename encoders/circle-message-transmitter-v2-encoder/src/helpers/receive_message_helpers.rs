use {
    super::FeeRecipientFetcher,
    crate::{TOKEN_MINTER_PROGRAM_ID, instructions::receive_message, types::ReceiveMessageParams},
    alloy_primitives::FixedBytes,
    borsh::BorshDeserialize,
    nitrogen_instruction_builder::{InstructionBuilder, derive_pda},
    solana_instruction::AccountMeta,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_rpc_client_api::client_error::Result as ClientResult,
    std::fmt::{Debug, Display},
    tracing::debug,
};

#[async_trait::async_trait]
impl<T: AsRef<RpcClient> + Send + Sync> FeeRecipientFetcher for T {
    async fn get_fee_recipient_token_account(
        &self,
        circle_usdc_address: &Pubkey,
    ) -> ClientResult<Pubkey> {
        let token_messenger_account =
            Pubkey::find_program_address(&[b"token_messenger"], &super::TOKEN_MINTER_PROGRAM_ID).0;
        debug!("lookup up token_messenger_account {token_messenger_account}");
        let account_data = self
            .as_ref()
            .get_account_data(&token_messenger_account)
            .await?;
        let fee_recipient = decode_fee_recipient_account(&account_data)?;
        let token_account = spl_associated_token_account::get_associated_token_address(
            &fee_recipient,
            circle_usdc_address,
        );
        debug!("fee recipient token account {token_account}");
        Ok(token_account)
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

impl Display for TokenMessenger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "denylister={},owner={},pending_owner={},message_body_version={},fee_recipient={},\
             min_fee_controller={},min_fee={}",
            self.denylister,
            self.owner,
            self.pending_owner,
            self.message_body_version,
            self.fee_recipient,
            self.min_fee_controller,
            self.min_fee
        )
    }
}

#[allow(clippy::result_large_err)]
pub fn decode_fee_recipient_account(account_data: &[u8]) -> ClientResult<Pubkey> {
    let messenger = TokenMessenger::try_from_slice(&account_data[8..])?;
    debug!("TokenMessenger: {messenger}");
    Ok(messenger.fee_recipient)
}

pub async fn fee_recipient_token_account<T: FeeRecipientFetcher>(
    rpc: &T,
    circle_usdc_address: &Pubkey,
) -> ClientResult<Pubkey> {
    rpc.get_fee_recipient_token_account(circle_usdc_address)
        .await
}

pub fn recv_from_attestation(
    destination_owner: Pubkey,
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
        TOKEN_MINTER_PROGRAM_ID,
        crate::ID,
    )
}

/// <https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/examples/v2/utilsV2.ts>
/// <https://github.com/circlefin/solana-cctp-block_oncontracts/blob/master/programs/v2/message-transmitter-v2/src/instructions/receive_message.rs>
/// <https://developers.circle.com/cctp/solana-programs#messagetransmitterv2>
pub fn remaining_accounts(
    recipient: &Pubkey,
    remote_domain: String,
    remote_address: FixedBytes<32>,
    circle_usdc_address: &Pubkey,
    fee_recipient_token_account: &Pubkey,
) -> Vec<AccountMeta> {
    let user_token_account =
        spl_associated_token_account::get_associated_token_address(recipient, circle_usdc_address);
    let remote_address = Pubkey::new_from_array(remote_address.into());
    vec![
        derive_pda(&[b"token_messenger"], &TOKEN_MINTER_PROGRAM_ID, true),
        derive_pda(
            &[b"remote_token_messenger", remote_domain.as_bytes()],
            &TOKEN_MINTER_PROGRAM_ID,
            true,
        ),
        derive_pda(&[b"token_minter"], &TOKEN_MINTER_PROGRAM_ID, false),
        derive_pda(
            &[b"local_token", circle_usdc_address.as_ref()],
            &TOKEN_MINTER_PROGRAM_ID,
            false,
        ),
        derive_pda(
            &[
                b"token_pair",
                remote_domain.as_bytes(),
                remote_address.as_ref(),
            ],
            &TOKEN_MINTER_PROGRAM_ID,
            true,
        ),
        AccountMeta::new(*fee_recipient_token_account, false),
        AccountMeta::new(user_token_account, false),
        derive_pda(
            &[b"custody", circle_usdc_address.as_ref()],
            &TOKEN_MINTER_PROGRAM_ID,
            false,
        ),
        AccountMeta::new_readonly(spl_token::ID, false),
        derive_pda(&[b"__event_authority"], &TOKEN_MINTER_PROGRAM_ID, true),
        AccountMeta::new_readonly(TOKEN_MINTER_PROGRAM_ID, false),
    ]
}
