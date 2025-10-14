use {
    crate::accounts::MessageSent,
    solana_pubkey::Pubkey,
    std::{
        fmt::{Display, Formatter},
        time::Duration,
    },
};

pub struct ReclaimAccountStatus {
    pub owner: Pubkey,
    pub accounts: Vec<ReclaimAccount>,
}

impl Display for ReclaimAccountStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let accounts = self
            .accounts
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>();
        let balance: u64 = self.accounts.iter().map(|a| a.balance).sum();
        write!(
            f,
            "Owner: {} Total Reclaim Balance {}  Accounts: {}\n{}",
            self.owner,
            balance,
            self.accounts.len(),
            accounts.join("\n")
        )
    }
}

impl ReclaimAccountStatus {
    fn new(owner: Pubkey) -> Self {
        Self {
            owner,
            accounts: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ReclaimAccount {
    pub account: MessageSent,
    pub address: Pubkey,
    pub event_window_remaining: i64,
    pub balance: u64,
    pub signature: Option<String>,
}

impl Display for ReclaimAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let remaining_time = if self.event_window_remaining > 0 {
            format!(
                "remaining {}",
                humantime::format_duration(Duration::new(
                    self.event_window_remaining.unsigned_abs(),
                    0
                ))
            )
        } else {
            format!(
                "expired {}",
                humantime::format_duration(Duration::new(
                    self.event_window_remaining.unsigned_abs(),
                    0
                ))
            )
        };
        write!(
            f,
            "{} time {remaining_time} created at {} balance {}",
            self.address, self.account.created_at, self.balance
        )
    }
}

impl ReclaimAccount {
    fn new(
        address: Pubkey,
        account: MessageSent,
        event_window_remaining: i64,
        balance: u64,
        signature: Option<String>,
    ) -> ReclaimAccount {
        Self {
            address,
            account,
            event_window_remaining,
            balance,
            signature,
        }
    }

    pub fn is_claimable(&self) -> bool {
        self.event_window_remaining <= 0
    }
}

pub mod reclaim_event_account_helpers {
    use {
        crate::{ReclaimAccountStatus, accounts::MessageSent},
        base64::prelude::*,
        borsh::BorshDeserialize,
        solana_account_decoder_client_types::UiAccountEncoding,
        solana_client::{
            rpc_client::GetConfirmedSignaturesForAddress2Config,
            rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        },
        solana_pubkey::Pubkey,
        solana_rpc_client::nonblocking::rpc_client::RpcClient,
        solana_rpc_client_api::client_error::Result as ClientResult,
    };

    // https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/programs/v2/message-transmitter-v2/src/state.rs#L56
    const EVENT_ACCOUNT_WINDOW_SECONDS: i64 = 60 * 60 * 24 * 5; // 5 days

    pub async fn find_claimable_accounts(
        owner: &Pubkey,
        rpc: &RpcClient,
    ) -> ClientResult<ReclaimAccountStatus> {
        let program_id = crate::ID;
        let matcher = BASE64_STANDARD.encode(owner.to_bytes());
        let accounts = rpc
            .get_program_accounts_with_config(&program_id, RpcProgramAccountsConfig {
                filters: Some(vec![solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new(
                        8, // offset after discriminator
                        solana_client::rpc_filter::MemcmpEncodedBytes::Base64(matcher),
                    ),
                )]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await?;
        let mut claimable = ReclaimAccountStatus::new(*owner);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        for (pubkey, account) in accounts {
            // Parse created_at from account data (offset 40: 8 discriminator + 32
            // rent_payer)
            if account.data.len() >= 48 {
                // Skip 8-byte Anchor discriminator
                let data_without_discriminator = &account.data[8..];
                // Deserialize with borsh
                let result = MessageSent::try_from_slice(data_without_discriminator);
                if let Err(e) = result {
                    tracing::warn!("failed to deserialize message sent: {e} on pubkey {pubkey}");
                    continue;
                }
                let message_sent = result.unwrap_or_default();
                let time_remaining =
                    (message_sent.created_at + EVENT_ACCOUNT_WINDOW_SECONDS).saturating_sub(now);

                claimable.accounts.push(crate::ReclaimAccount::new(
                    pubkey,
                    message_sent,
                    time_remaining,
                    account.lamports,
                    get_reclaim_account_transaction(rpc, &pubkey).await?,
                ))
            }
        }

        Ok(claimable)
    }

    pub async fn get_reclaim_account_transaction(
        rpc: &RpcClient,
        reclaim_account_address: &Pubkey,
    ) -> ClientResult<Option<String>> {
        let result = rpc
            .get_signatures_for_address_with_config(
                reclaim_account_address,
                GetConfirmedSignaturesForAddress2Config {
                    limit: Some(1),
                    ..Default::default()
                },
            )
            .await?;
        if result.is_empty() {
            return Ok(None);
        }
        let sig = result[0].signature.clone();
        Ok(Some(sig))
    }
}

pub mod receive_message_helpers {
    use {
        crate::{instructions::receive_message, types::ReceiveMessageParams},
        alloy_primitives::FixedBytes,
        borsh::BorshDeserialize,
        nitrogen_instruction_builder::{InstructionBuilder, derive_pda},
        solana_instruction::AccountMeta,
        solana_pubkey::Pubkey,
        solana_rpc_client::nonblocking::rpc_client::RpcClient,
        solana_rpc_client_api::client_error::Result as ClientResult,
    };

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

    pub async fn fee_recipient_token_account(
        rpc: &RpcClient,
        token_minter_program: &Pubkey,
        circle_usdc_address: &Pubkey,
    ) -> ClientResult<Pubkey> {
        let token_messenger_account =
            Pubkey::find_program_address(&[b"token_messenger"], token_minter_program).0;
        let data = rpc.get_account_data(&token_messenger_account).await?;
        let fee_recipient = TokenMessenger::try_from_slice(&data[8..])?.fee_recipient;
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
        let user_token_account = spl_associated_token_account::get_associated_token_address(
            recipient,
            circle_usdc_address,
        );
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
}
