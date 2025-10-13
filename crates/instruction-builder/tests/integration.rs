#[cfg(not(feature = "blocking"))]
mod tests {
use {
    borsh::BorshSerialize,
    nitrogen_instruction_builder::{InstructionBuilder, TransactionBuilder},
    solana_commitment_config::CommitmentConfig,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
    std::{env, sync::Once},
    tracing::info,
    tracing_subscriber::{EnvFilter, fmt::format::FmtSpan},
};
pub static INIT: Once = Once::new();

#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
pub fn setup() {
    INIT.call_once(|| {
        if env::var("CI").is_err() {
            // only load .env if not in CI
            if dotenvy::dotenv_override().is_err() {
                eprintln!("no .env file");
            }
        }
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    });
}

fn init() -> anyhow::Result<(Keypair, RpcClient)> {
    setup();
    let kp_file = env::var("KEYPAIR_FILE").ok();
    let owner = if let Some(kp) = kp_file {
        solana_keypair::read_keypair_file(&kp).expect(&format!(
            "unable to load
    keypair file {kp}"
        ))
    } else {
        let kp = env::var("KEYPAIR").expect("KEYPAIR is not set");
        Keypair::from_base58_string(&kp)
    };
    info!("using solana address {}", owner.pubkey());
    let url = env::var("RPC_URL").expect("RPC_URL is not set");
    info!("using RPC {url}");
    let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::finalized());
    Ok((owner, rpc))
}

#[derive(BorshSerialize)]
struct MemoData {
    pub memo: Vec<u8>,
}

impl Into<MemoData> for &str {
    fn into(self) -> MemoData {
        MemoData {
            memo: self.to_string().into_bytes(),
        }
    }
}
impl Into<MemoData> for String {
    fn into(self) -> MemoData {
        MemoData {
            memo: self.into_bytes(),
        }
    }
}

#[tokio::test]
async fn test_instruction_builder() -> anyhow::Result<()> {
    let (kp, rpc) = init()?;
    let memo: MemoData = "Hello, World!".into();
    let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];
    let b = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts)
        .params(memo)
        .build();
    let sig = b.tx().send(&rpc, Some(&kp.pubkey()), &[&kp]).await?;
    info!("{sig}");
    Ok(())
}

#[tokio::test]
async fn test_transaction_builder_single_instruction() -> anyhow::Result<()> {
    let (kp, rpc) = init()?;
    let memo: MemoData = "Single instruction test".into();
    let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];

    let instruction_builder = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts)
        .params(memo)
        .build();

    let tx = TransactionBuilder::builder()
        .instructions(vec![])
        .build()
        .push(instruction_builder);

    let sig = tx.send(&rpc, Some(&kp.pubkey()), &[&kp]).await?;
    info!("Single instruction tx: {sig}");
    Ok(())
}

#[tokio::test]
async fn test_transaction_builder_multiple_instructions() -> anyhow::Result<()> {
    let (kp, rpc) = init()?;
    let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];

    let memo1: MemoData = "First memo".into();
    let memo2: MemoData = "Second memo".into();
    let memo3: MemoData = "Third memo".into();

    let builders = vec![
        InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts.clone())
            .params(memo1)
            .build(),
        InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts.clone())
            .params(memo2)
            .build(),
        InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts.clone())
            .params(memo3)
            .build(),
    ];

    let tx = TransactionBuilder::builder()
        .instructions(vec![])
        .build()
        .append(builders);

    let sig = tx.send(&rpc, Some(&kp.pubkey()), &[&kp]).await?;
    info!("Multiple instructions tx: {sig}");
    Ok(())
}

#[test]
fn test_remaining_accounts() {
    let memo: MemoData = "With remaining accounts".into();
    let base_accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];
    let remaining_accounts = vec![
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
    ];

    let instruction_builder = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(base_accounts.clone())
        .params(memo)
        .build()
        .remaining_accounts(remaining_accounts.clone());

    let instruction = instruction_builder.instruction();

    // Verify the instruction has all accounts (base + remaining)
    assert_eq!(
        instruction.accounts.len(),
        base_accounts.len() + remaining_accounts.len()
    );
    assert_eq!(instruction.program_id, spl_memo::id());
}

#[tokio::test]
async fn test_memo_data_conversions() -> anyhow::Result<()> {
    let (kp, rpc) = init()?;
    let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];

    // Test &str conversion
    let memo_str: MemoData = "String slice memo".into();
    let builder1 = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts.clone())
        .params(memo_str)
        .build();

    // Test String conversion
    let memo_string: MemoData = String::from("Owned string memo").into();
    let builder2 = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts.clone())
        .params(memo_string)
        .build();

    let tx = TransactionBuilder::builder()
        .instructions(vec![])
        .build()
        .push(builder1)
        .push(builder2);

    let sig = tx.send(&rpc, Some(&kp.pubkey()), &[&kp]).await?;
    info!("Memo conversions tx: {sig}");
    Ok(())
}

#[tokio::test]
async fn test_empty_memo() -> anyhow::Result<()> {
    let (kp, rpc) = init()?;
    let memo: MemoData = "".into();
    let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];

    let instruction_builder = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts)
        .params(memo)
        .build();

    let sig = instruction_builder
        .tx()
        .send(&rpc, Some(&kp.pubkey()), &[&kp])
        .await?;
    info!("Empty memo tx: {sig}");
    Ok(())
}

#[test]
fn test_instruction_creation() {
    let memo: MemoData = "Test instruction creation".into();
    let accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];

    let builder = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts.clone())
        .params(memo)
        .build();

    let instruction = builder.instruction();
    assert_eq!(instruction.program_id, spl_memo::id());
    assert_eq!(instruction.accounts.len(), accounts.len());
}

#[test]
fn test_transaction_builder_creation() {
    let memo: MemoData = "Test transaction creation".into();
    let accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];

    let builder = InstructionBuilder::builder()
        .program_id(spl_memo::id())
        .accounts(accounts)
        .params(memo)
        .build();

    let tx = builder.tx();
    assert_eq!(tx.instructions.len(), 1);
    assert_eq!(tx.instructions[0].program_id, spl_memo::id());
}

#[test]
fn test_memo_data_struct() {
    let memo_from_str: MemoData = "Hello".into();
    let memo_from_string: MemoData = String::from("World").into();

    assert_eq!(memo_from_str.memo, b"Hello");
    assert_eq!(memo_from_string.memo, b"World");
}
}
