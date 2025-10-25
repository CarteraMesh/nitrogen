use {
    crate::command::BurnArgs,
    alloy_primitives::Address,
    anyhow::{Ok, Result},
    clap::Parser,
    nitrogen_circle_message_transmitter_v2_encoder::{
        ID as MESSAGE_TRANSMITTER_PROGRAM_ID,
        helpers::{receive_message_helpers, reclaim_event_account_helpers},
    },
    nitrogen_circle_token_messenger_minter_v2_encoder::{
        ID as TOKEN_MINTER_PROGRAM_ID,
        helpers::{SOLANA_DEV_USDC_ADDRESS as SOLANA_USDC_ADDRESS, deposit_for_burn_instruction},
        types::DepositForBurnParams,
    },
    solana_commitment_config::CommitmentConfig,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_rpc_client_api::config::RpcSimulateTransactionConfig,
    solana_signer::Signer,
    soly::{
        BlockHashCacheProvider,
        LookupTableCacheProvider,
        SimpleCacheTransactionProvider,
        TraceTransactionArcProvider,
        TransactionBuilder,
        TransactionRpcProvider,
    },
    std::{env, sync::Arc, time::Duration},
    tracing::{Level, info, span},
};
mod attestation;
mod command;

pub fn memo(message: &str) -> Instruction {
    Instruction {
        program_id: spl_memo::id(),
        accounts: vec![],
        data: message.as_bytes().to_vec(),
    }
}
async fn fetch_attestation(
    sig: String,
    chain: Option<u32>,
) -> Result<(attestation::AttestationBytes, attestation::AttestationBytes)> {
    // Fetch attestation logic here
    attestation::get_attestation_with_retry(sig, chain).await
}

async fn reclaim<T: TransactionRpcProvider + AsRef<RpcClient>>(
    rpc: &T,
    owner: Keypair,
) -> Result<()> {
    let reclaim_accounts =
        reclaim_event_account_helpers::find_claimable_accounts(&owner.pubkey(), rpc).await?;
    info!("reclaim accounts {reclaim_accounts}");
    let mut fee: Option<u64> = None;
    let mut units: Option<u32> = None;
    for account in reclaim_accounts.accounts {
        let span = span!(
            Level::INFO,
            "reclaim-account",
            acct = ?account.address,
        );
        let _guard = span.enter();
        if !account.is_claimable() {
            info!("Skipping account");
            continue;
        }
        if account.signature.is_none() {
            tracing::warn!("Skipping account with no signature");
            continue;
        }
        let sig = account.signature.clone().unwrap();
        let (attest, message) = fetch_attestation(sig, None).await?;
        let reclaim_tx: TransactionBuilder = account.instruction((attest, message)).into();
        let reclaim_tx = match (units, fee) {
            (Some(units), Some(fee)) => {
                reclaim_tx.prepend_compute_budget_instructions(units, fee)?
            }
            (Some(_), None) => {
                return Err(anyhow::format_err!(
                    "fee and units should be both none or some"
                ));
            }
            (None, Some(_)) => {
                return Err(anyhow::format_err!(
                    "fee and units should be both none or some"
                ));
            }
            (None, None) => {
                let fee_result = reclaim_tx
                    .calc_fee(
                        &owner.pubkey(),
                        rpc,
                        &[
                            MESSAGE_TRANSMITTER_PROGRAM_ID,
                            solana_system_interface::program::ID,
                        ],
                        60_000_000,
                        Some(50),
                    )
                    .await?;
                fee = Some(fee_result.priority_fee);
                units = Some((fee_result.units + 10_000).min(1_400_000)); //max units
                reclaim_tx.prepend_compute_budget_instructions(units.unwrap(), fee.unwrap())?
            }
        };

        info!("reclaiming");
        let result = reclaim_tx
            .simulate(
                &owner.pubkey(),
                &[&owner],
                rpc,
                RpcSimulateTransactionConfig {
                    sig_verify: true,
                    ..Default::default()
                },
            )
            .await?;
        info!(
            "compute units {}",
            result.units_consumed.unwrap_or_default()
        );
    }
    Ok(())
}

fn get_keypair() -> Result<Keypair> {
    let kp_file = env::var("KEYPAIR_FILE").ok();
    let owner = if let Some(kp) = kp_file {
        solana_keypair::read_keypair_file(&kp)
            .map_err(|e| anyhow::format_err!("failed to read file {e}"))?
    } else {
        let kp = env::var("TEST_PRIVATE_KEY")
            .map_err(|_| anyhow::format_err!("TEST_PRIVATE_KEY environment variable not set"))?;
        Keypair::from_base58_string(&kp)
    };
    Ok(owner)
}

#[allow(unused_variables)]
async fn evm_sol<T: TransactionRpcProvider>(args: BurnArgs, owner: Keypair, rpc: T) -> Result<()> {
    info!("TBD sending to sol");
    Ok(())
}

async fn sol_evm<T: TransactionRpcProvider>(args: BurnArgs, owner: Keypair, rpc: T) -> Result<()> {
    let span = tracing::info_span!("sol_evm", args =? args);
    let _g = span.enter();
    info!("burning...");
    let message_sent_event_account = Keypair::new();
    let evm_addr: Address = Address::parse_checksummed(args.destination, None)?;
    // mintRecipient is a bytes32 type so pad with 0's then convert to a
    // solana PublicKey
    let mint_recipient = Pubkey::new_from_array(evm_addr.into_word().into());
    let params = DepositForBurnParams::builder()
        .amount(args.amount)
        .destination_caller(Pubkey::default())
        .mint_recipient(mint_recipient)
        .max_fee(3)
        .min_finality_threshold(0)
        .destination_domain(args.destination_chain)
        .build();
    info!("Params\n{:?}", params);
    let deposit_for_burn_tx = deposit_for_burn_instruction(
        params,
        owner.pubkey(),
        message_sent_event_account.pubkey(),
        SOLANA_USDC_ADDRESS,
    );
    for (i, a) in deposit_for_burn_tx.accounts.iter().enumerate() {
        eprintln!(
            "[{}]    {},signer={},mut={}",
            i + 1,
            a.pubkey,
            a.is_signer,
            a.is_writable
        );
    }
    let tx = TransactionBuilder::from(deposit_for_burn_tx.instruction())
        .with_memo("github.com/carteraMesh/nitrogen", &[&owner.pubkey()])
        .with_priority_fees(
            &owner.pubkey(),
            &rpc,
            &[TOKEN_MINTER_PROGRAM_ID, spl_token::ID],
            60_000_000,
            Some(50),
        )
        .await?;
    let sig = tx
        .send(&rpc, &owner.pubkey(), &[
            &owner,
            &message_sent_event_account,
        ])
        .await?;
    info!("{sig}");
    Ok(())
}

fn cached_rpc(rpc: RpcClient) -> impl TransactionRpcProvider + AsRef<RpcClient> {
    let rpc: TraceTransactionArcProvider = Arc::new(rpc).into();
    let hash_cache = BlockHashCacheProvider::new(rpc.clone(), Duration::from_secs(30));
    let lookup_cache = LookupTableCacheProvider::builder()
        .inner(rpc.clone())
        .lookup_cache(
            soly::Cache::builder()
                .time_to_live(Duration::from_secs(86400))
                .build(),
        )
        .negative_cache(
            soly::Cache::builder()
                .time_to_live(Duration::from_secs(600))
                .build(),
        )
        .build();

    SimpleCacheTransactionProvider::builder()
        .inner(rpc.clone())
        .blockhash_cache(hash_cache.into())
        .lookup_cache(lookup_cache.into())
        .build()
}

#[allow(clippy::expect_fun_call)]
#[tokio::main]
pub async fn main() -> Result<()> {
    dotenvy::dotenv_override().ok();
    tracing_subscriber::fmt()
//             .with_target(true)
             .with_level(true)
             .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
             .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
             .init();
    let cli = command::Cli::parse();
    let owner = get_keypair()?;

    info!("using solana address {}", owner.pubkey());

    let url = env::var("RPC_URL").expect("RPC_URL is not set");
    info!("using RPC {url}");
    let rpc = cached_rpc(RpcClient::new_with_commitment(
        url,
        CommitmentConfig::finalized(),
    ));
    match cli.command {
        command::Commands::Burn(args) => {
            if !args.to_sol {
                sol_evm(args, owner, rpc).await
            } else {
                evm_sol(args, owner, rpc).await
            }
        }
        command::Commands::Reclaim => {
            reclaim(&rpc, owner).await?;
            Ok(())
        }
        command::Commands::Recv { tx_hash } => {
            info!("recv for {tx_hash}");
            let (attest, message) = fetch_attestation(tx_hash, Some(6)).await?;
            info!(
                "attestation: {}\nmessage: {}",
                alloy_primitives::hex::encode(&attest),
                alloy_primitives::hex::encode(&message),
            );
            let builder = receive_message_helpers::recv_from_attestation(
                owner.pubkey(),
                TOKEN_MINTER_PROGRAM_ID,
                attest,
                message,
            );
            let fee_recipient =
                receive_message_helpers::fee_recipient_token_account(&rpc, &SOLANA_USDC_ADDRESS)
                    .await?;
            let usdc_evm_addr: Address =
                alloy_primitives::address!("0x036CbD53842c5426634e7929541eC2318f3dCF7e"); // base sepolia
            let remaining_accounts = receive_message_helpers::remaining_accounts(
                &owner.pubkey(),
                "6".to_string(), // base sepolia
                usdc_evm_addr.into_word(),
                &TOKEN_MINTER_PROGRAM_ID,
                &SOLANA_USDC_ADDRESS,
                &fee_recipient,
            );
            let builder = builder.remaining_accounts(remaining_accounts);
            for (i, a) in builder.accounts.iter().enumerate() {
                eprintln!(
                    "[{}]    {},signer={},mut={}",
                    i + 1,
                    a.pubkey,
                    a.is_signer,
                    a.is_writable
                );
            }
            let tx = TransactionBuilder::from(builder.instruction());
            let sig = tx.send(&rpc, &owner.pubkey(), &[&owner]).await?;
            info!("Transaction signature: {}", sig);
            Ok(())
        }
    }
}
