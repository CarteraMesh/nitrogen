use {
    alloy_primitives::Address,
    anyhow::Result,
    clap::Parser,
    nitrogen_circle_message_transmitter_v2_encoder::{
        ID as MESSENGER_PROGRAM_ID,
        helpers::{fee_recipient_token_account, recv_from_attestation, remaining_accounts},
        instructions::reclaim_event_account,
        types::ReclaimEventAccountParams,
    },
    nitrogen_circle_token_messenger_minter_v2_encoder::{
        ID as TOKEN_MINTER_PROGRAM_ID,
        instructions::deposit_for_burn,
        types::DepositForBurnParams,
    },
    nitrogen_instruction_builder::IntoInstruction,
    solana_commitment_config::CommitmentConfig,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::{Pubkey, pubkey},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
    std::env,
    tracing::info,
};
mod attestation;
mod command;
const SOLANA_USDC_ADDRESS: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

pub fn memo(message: &str) -> Instruction {
    Instruction {
        program_id: spl_memo::id(),
        accounts: vec![],
        data: message.as_bytes().to_vec(),
    }
}
fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program_id).0
}

async fn fetch_attestation(
    sig: String,
    chain: Option<u32>,
) -> Result<(attestation::AttestationBytes, attestation::AttestationBytes)> {
    // Fetch attestation logic here
    attestation::get_attestation_with_retry(sig, chain).await
}

async fn reclaim(
    rpc: &RpcClient,
    tx_hash: String,
    owner: Keypair,
    message_sent_event_account: Pubkey,
) -> Result<()> {
    let (attest, message) = fetch_attestation(tx_hash, None).await?;
    log::info!(
        "attestation: {}\nmessage: {}",
        alloy_primitives::hex::encode(&attest),
        alloy_primitives::hex::encode(&message),
    );

    let reclaim_account = reclaim_event_account(
        ReclaimEventAccountParams::builder()
            .attestation(attest)
            .destination_message(message)
            .build(),
    );

    let reclaim_tx = reclaim_account
        .accounts(
            owner.pubkey(),
            derive_pda(&[b"message_transmitter"], &MESSENGER_PROGRAM_ID),
            message_sent_event_account,
        )
        .tx()
        .push(
            spl_memo::build_memo("github.com/carteraMesh/nitrogen".as_bytes(), &[
                &owner.pubkey()
            ])
            .into_instruction(),
        );

    let sig = reclaim_tx
        .send(rpc, Some(&owner.pubkey()), &[&owner])
        .await?;
    log::info!("{sig}");
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

#[allow(clippy::expect_fun_call)]
#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let cli = command::Cli::parse();
    let owner = get_keypair()?;
    let message_sent_event_account = Keypair::new();

    log::info!("using solana address {}", owner.pubkey());

    let url = env::var("RPC_URL").expect("RPC_URL is not set");
    log::info!("using RPC {url}");
    let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::finalized());
    match cli.command {
        command::Commands::Bridge {
            amount,
            destination_chain,
            destination,
        } => {
            log::info!("burning...");
            let evm_addr: Address = Address::parse_checksummed(destination, None)?;
            // mintRecipient is a bytes32 type so pad with 0's then convert to a
            // solana PublicKey
            let mint_recipient = Pubkey::new_from_array(evm_addr.into_word().into());
            let deposit_for_burn = deposit_for_burn(
                DepositForBurnParams::builder()
                    .amount(amount)
                    .destination_caller(Pubkey::default())
                    .mint_recipient(mint_recipient)
                    .max_fee(3)
                    .min_finality_threshold(0)
                    .destination_domain(destination_chain)
                    .build(),
            );

            eprintln!("amount: {}", deposit_for_burn.params.amount);
            eprintln!(
                "destination: {}",
                deposit_for_burn.params.destination_domain
            );
            eprintln!("mint recipient: {}", deposit_for_burn.params.mint_recipient);
            eprintln!("maxFee: {}", deposit_for_burn.params.max_fee);
            eprintln!(
                "minFinalityThreshold: {}",
                deposit_for_burn.params.min_finality_threshold
            );
            eprintln!(
                "destinationCaller: {}",
                deposit_for_burn.params.destination_caller
            );
            // log::info!(
            //     "Send to {mint_recipient} on domain {DOMAIN} params:\n{}",
            //     serde_json::to_string_pretty(&deposit_for_burn.params)?
            // );
            let owner_token_account = spl_associated_token_account::get_associated_token_address(
                &owner.pubkey(),
                &SOLANA_USDC_ADDRESS,
            );
            log::info!("token account {owner_token_account}");
            let burn_inx = deposit_for_burn.accounts(
                owner.pubkey(),
                owner.pubkey(),
                owner_token_account,
                derive_pda(&[b"message_transmitter"], &MESSENGER_PROGRAM_ID),
                derive_pda(&[b"token_messenger"], &TOKEN_MINTER_PROGRAM_ID),
                derive_pda(&[b"remote_token_messenger", b"6"], &TOKEN_MINTER_PROGRAM_ID),
                derive_pda(&[b"token_minter"], &TOKEN_MINTER_PROGRAM_ID),
                SOLANA_USDC_ADDRESS,
                message_sent_event_account.pubkey(),
                TOKEN_MINTER_PROGRAM_ID,
            );
            for (i, a) in burn_inx.accounts.iter().enumerate() {
                eprintln!(
                    "[{}]    {},signer={},mut={}",
                    i + 1,
                    a.pubkey,
                    a.is_signer,
                    a.is_writable
                );
            }
            let tx = burn_inx.tx().push(spl_memo::build_memo(
                "github.com/carteraMesh/nitrogen".as_bytes(),
                &[&owner.pubkey()],
            ));
            let sig = tx
                .send(&rpc, Some(&owner.pubkey()), &[
                    &owner,
                    &message_sent_event_account,
                ])
                .await?;
            log::info!("{sig}");
            Ok(())
        }
        command::Commands::Reclaim {
            message_sent_event_account,
            tx_hash,
        } => {
            reclaim(&rpc, tx_hash, owner, message_sent_event_account).await?;
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
            let builder =
                recv_from_attestation(owner.pubkey(), TOKEN_MINTER_PROGRAM_ID, attest, message);
            let fee_recipient =
                fee_recipient_token_account(&rpc, &TOKEN_MINTER_PROGRAM_ID, &SOLANA_USDC_ADDRESS)
                    .await?;
            let usdc_evm_addr: Address =
                alloy_primitives::address!("0x036CbD53842c5426634e7929541eC2318f3dCF7e"); // base sepolia
            let remaining_accounts = remaining_accounts(
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
            let sig = tx.send(&rpc, Some(&owner.pubkey()), &[&owner]).await?;
            info!("Transaction signature: {}", sig);
            Ok(())
        }
    }
}
