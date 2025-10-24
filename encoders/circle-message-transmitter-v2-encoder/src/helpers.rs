use {
    crate::{accounts::MessageSent, types::ReclaimEventAccountParams},
    solana_account::Account,
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_pubkey::Pubkey,
    solana_rpc_client_api::client_error::Result as ClientResult,
    std::{
        fmt::{Display, Formatter},
        time::Duration,
    },
};

pub const TOKEN_MINTER_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("CCTPV2vPZJS2u2BBsUoscuikbYjnpFmbFsvVuJdgUMQe");

#[async_trait::async_trait]
pub trait ReclaimAccountRpcState: Send + Sync {
    async fn get_reclaim_accounts(&self, owner: &Pubkey) -> ClientResult<Vec<(Pubkey, Account)>>;
    async fn get_reclaim_account_signature(&self, account: &Pubkey)
    -> ClientResult<Option<String>>;
}

/// The fee account is owned by Circle and is used to pay for CCTP fees.
#[async_trait::async_trait]
pub trait FeeRecipientFetcher: Send + Sync {
    async fn get_fee_recipient_token_account(
        &self,
        circle_usdc_address: &Pubkey,
    ) -> ClientResult<Pubkey>;
}

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
/// Circle CCTP stores an attestation and a message. This is used to verify your
/// claim to USDC
///
/// See `curl https://iris-api-sandbox.circle.com/v2/messages/6/\?transactionHash\=0x1de765f7d19b45913190863d8cd60c1e58e48a85b60b0ff7bf39329076aabd7b  --header  'Content-Type: application/json'  | jq . ` for example
pub type AttestationMessage = (Vec<u8>, Vec<u8>);

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

    /// Constructs an instruction to reclaim the event account used to burn
    /// USDC.
    ///
    /// See <https://developers.circle.com/cctp/solana-programs>
    pub fn instruction(
        &self,
        attestation_message: AttestationMessage,
    ) -> solana_instruction::Instruction {
        crate::instructions::reclaim_event_account(
            ReclaimEventAccountParams::builder()
                .attestation(attestation_message.0)
                .destination_message(attestation_message.1)
                .build(),
        )
        .accounts(self.account.rent_payer, self.address)
        .instruction()
    }
}

pub mod receive_message_helpers;
pub mod reclaim_event_account_helpers;
