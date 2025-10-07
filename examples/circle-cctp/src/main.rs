use {
    alloy_primitives::Address,
    anyhow::Result,
    base64::prelude::*,
    nitrogen_circle_message_transmitter_v2_encoder::ID as MESSENGER_PROGRAM_ID,
    nitrogen_circle_token_messenger_minter_v2_encoder::{
        ID as PROGRAM_ID,
        instructions::deposit_for_burn::DepositForBurn,
        types::DepositForBurnParams,
    },
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::{Pubkey, pubkey},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_rpc_client_api::config::RpcSimulateTransactionConfig,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::env,
};

const SOLANA_USDC_ADDRESS: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
const DOMAIN: u32 = 6;
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

#[allow(clippy::expect_fun_call)]
#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let kp_file = env::var("KEYPAIR_FILE").expect("KEYPAIR_FILE environment variable not set");
    let owner = solana_keypair::read_keypair_file(&kp_file)
        .expect(&format!("unable to load keypair file {kp_file}"));
    let message_sent_event_account = Keypair::new();
    let evm_addr: Address = Address::parse_checksummed(
        env::var("DESTINATION").expect("DESTINATION environment variable not set"),
        None,
    )?;
    log::info!("using solana address {}", owner.pubkey());
    // mintRecipient is a bytes32 type so pad with 0's then convert to a solana
    // PublicKey
    let mint_recipient = Pubkey::new_from_array(evm_addr.into_word().into());
    let deposit_for_burn = DepositForBurn {
        params: DepositForBurnParams {
            amount: 1,
            destination_domain: DOMAIN, // base
            mint_recipient,
            max_fee: 0,
            min_finality_threshold: 0,
            destination_caller: Pubkey::default(),
        },
    };

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
    let burn_inx = deposit_for_burn.build(
        owner.pubkey(),
        owner.pubkey(),
        owner_token_account,
        derive_pda(&[b"message_transmitter"], &MESSENGER_PROGRAM_ID),
        derive_pda(&[b"token_messenger"], &PROGRAM_ID),
        derive_pda(&[b"remote_token_messenger", b"6"], &PROGRAM_ID),
        derive_pda(&[b"token_minter"], &PROGRAM_ID),
        SOLANA_USDC_ADDRESS,
        message_sent_event_account.pubkey(),
        PROGRAM_ID,
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

    let url = env::var("RPC_URL").expect("RPC_URL is not set");
    log::info!("using RPC {url}");
    let rpc = RpcClient::new(url);
    let hash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[
            burn_inx,
            spl_memo::build_memo("github.com/carteraMesh".as_bytes(), &[&owner.pubkey()]),
        ],
        Some(&owner.pubkey()),
        &[&owner, &message_sent_event_account],
        hash,
    );
    let transaction_base64 = BASE64_STANDARD.encode(bincode::serialize(&tx)?);
    eprintln!("{transaction_base64}");
    let result = rpc
        .simulate_transaction_with_config(&tx, RpcSimulateTransactionConfig {
            sig_verify: true,
            ..RpcSimulateTransactionConfig::default()
        })
        .await?;

    if let Some(e) = result.value.err {
        if let Some(logs) = result.value.logs {
            log::error!("Transaction failed with logs: {}", logs.join("\n"));
        }
        return Err(e.into());
    }
    let sig = rpc.send_and_confirm_transaction(&tx).await?;
    log::info!("tx {sig}");
    Ok(())
}
