use {
    crate::command::BridgeArgs,
    alloy_primitives::Address,
    anyhow::{Ok, Result},
    clap::Parser,
    nitrogen_circle_message_transmitter_v2_encoder::{
        helpers::{receive_message_helpers, reclaim_event_account_helpers},
        instructions::reclaim_event_account,
        types::ReclaimEventAccountParams,
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
    std::env,
    tracing::info,
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

async fn reclaim(rpc: &RpcClient, owner: Keypair) -> Result<()> {
    let reclaim_accounts =
        reclaim_event_account_helpers::find_claimable_accounts(&owner.pubkey(), rpc).await?;
    info!("reclaim accounts {reclaim_accounts}");
    for account in reclaim_accounts.accounts {
        if !account.is_claimable() {
            info!("Skipping account {account}");
            continue;
        }
        if account.signature.is_none() {
            tracing::warn!("Skipping account {account} with no signature");
            continue;
        }
        let sig = account.signature.unwrap_or_default();
        let (attest, message) = fetch_attestation(sig, None).await?;
        let reclaim_account = reclaim_event_account(
            ReclaimEventAccountParams::builder()
                .attestation(attest)
                .destination_message(message)
                .build(),
        );
        let reclaim_tx = reclaim_account
            .accounts(owner.pubkey(), account.address)
            .tx();

        info!("reclaiming {}", account.address);
        reclaim_tx
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
async fn evm_sol(args: BridgeArgs, owner: Keypair, rpc: RpcClient) -> Result<()> {
    info!("sending to sol");
    Ok(())
}

async fn sol_evm(args: BridgeArgs, owner: Keypair, rpc: RpcClient) -> Result<()> {
    log::info!("burning...");
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
    let tx = deposit_for_burn_tx.tx().push(spl_memo::build_memo(
        "github.com/carteraMesh/nitrogen".as_bytes(),
        &[&owner.pubkey()],
    ));
    let sig = tx
        .send(&rpc, &owner.pubkey(), &[
            &owner,
            &message_sent_event_account,
        ])
        .await?;
    log::info!("{sig}");
    Ok(())
}

#[allow(clippy::expect_fun_call)]
#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let cli = command::Cli::parse();
    let owner = get_keypair()?;

    log::info!("using solana address {}", owner.pubkey());

    let url = env::var("RPC_URL").expect("RPC_URL is not set");
    log::info!("using RPC {url}");
    let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::finalized());
    match cli.command {
        command::Commands::Bridge(args) => {
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
            log::info!(
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
            let fee_recipient = receive_message_helpers::fee_recipient_token_account(
                &rpc,
                &TOKEN_MINTER_PROGRAM_ID,
                &SOLANA_USDC_ADDRESS,
            )
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
            let tx = builder.tx();
            let sig = tx.send(&rpc, &owner.pubkey(), &[&owner]).await?;
            info!("Transaction signature: {}", sig);
            Ok(())
        }
    }
}
