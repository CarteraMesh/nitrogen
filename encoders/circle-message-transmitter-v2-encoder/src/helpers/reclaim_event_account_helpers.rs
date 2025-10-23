use {
    super::*,
    crate::{ReclaimAccountRpcState, ReclaimAccountStatus, accounts::MessageSent},
    base64::prelude::*,
    borsh::BorshDeserialize,
    solana_account_decoder_client_types::UiAccountEncoding,
    solana_client::{
        rpc_client::GetConfirmedSignaturesForAddress2Config,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    },
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

// https://github.com/circlefin/solana-cctp-contracts/blob/03f7dec786eb9affa68688954f62917edeed2e35/programs/v2/message-transmitter-v2/src/state.rs#L56
const EVENT_ACCOUNT_WINDOW_SECONDS: i64 = 60 * 60 * 24 * 5; // 5 days

#[async_trait::async_trait]
impl<T: AsRef<RpcClient> + Send + Sync> ReclaimAccountRpcState for T {
    async fn get_reclaim_accounts(&self, owner: &Pubkey) -> ClientResult<Vec<(Pubkey, Account)>> {
        let program_id = crate::ID;
        let matcher = BASE64_STANDARD.encode(owner.to_bytes());
        self.as_ref()
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
            .await
    }

    async fn get_reclaim_account_signature(
        &self,
        account: &Pubkey,
    ) -> ClientResult<Option<String>> {
        let result = self
            .as_ref()
            .get_signatures_for_address_with_config(
                account,
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

pub async fn find_claimable_accounts<T: ReclaimAccountRpcState>(
    owner: &Pubkey,
    rpc: &T,
) -> ClientResult<ReclaimAccountStatus> {
    let accounts = rpc.get_reclaim_accounts(owner).await?;
    let mut claimable = ReclaimAccountStatus::new(*owner);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    for (reclaim_address, account) in accounts {
        // Parse created_at from account data (offset 40: 8 discriminator + 32
        // rent_payer)
        if account.data.len() >= 48 {
            // Skip 8-byte Anchor discriminator
            let data_without_discriminator = &account.data[8..];
            // Deserialize with borsh
            let result = MessageSent::try_from_slice(data_without_discriminator);
            if let Err(e) = result {
                tracing::warn!(
                    "failed to deserialize message sent: {e} on reclaim_address {reclaim_address}"
                );
                continue;
            }
            let message_sent = result.unwrap_or_default();
            let time_remaining =
                (message_sent.created_at + EVENT_ACCOUNT_WINDOW_SECONDS).saturating_sub(now);

            claimable.accounts.push(crate::ReclaimAccount::new(
                reclaim_address,
                message_sent,
                time_remaining,
                account.lamports,
                rpc.get_reclaim_account_signature(&reclaim_address).await?,
            ))
        }
    }

    Ok(claimable)
}
