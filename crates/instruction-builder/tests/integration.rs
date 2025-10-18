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
                .with_target(true)
                .with_level(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        });
    }

    #[allow(clippy::expect_fun_call)]
    fn init() -> anyhow::Result<(Keypair, RpcClient)> {
        setup();
        let kp_file = env::var("KEYPAIR_FILE").ok();
        let owner = if let Some(kp) = kp_file {
            solana_keypair::read_keypair_file(&kp).expect(&format!(
                "unable to load
    keypair file {kp}"
            ))
        } else {
            let kp = env::var("TEST_PRIVATE_KEY").expect("TEST_PRIVATE_KEY is not set");
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

    impl From<&str> for MemoData {
        fn from(value: &str) -> Self {
            MemoData {
                memo: value.to_string().into_bytes(),
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
        let sig = b.tx().send(&rpc, &kp.pubkey(), &[&kp]).await?;
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

        let sig = tx.send(&rpc, &kp.pubkey(), &[&kp]).await?;
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

        let sig = tx.send(&rpc, &kp.pubkey(), &[&kp]).await?;
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
            .send(&rpc, &kp.pubkey(), &[&kp])
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
    fn test_extend_instruction() {
        let memo: MemoData = "Test extend".into();
        let accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];

        let ix = InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts)
            .params(memo)
            .build()
            .instruction();

        let mut tx = TransactionBuilder::default();
        tx.extend(vec![ix.clone(), ix.clone()]);

        assert_eq!(tx.instructions.len(), 2);
        assert_eq!(tx.instructions[0].program_id, spl_memo::id());
        assert_eq!(tx.instructions[1].program_id, spl_memo::id());
    }

    #[cfg(test)]
    #[cfg(not(feature = "blocking"))]
    mod lookups {
        use {
            super::*,
            nitrogen_instruction_builder::fetch_lookup_tables,
            solana_message::AddressLookupTableAccount,
            solana_pubkey::pubkey,
        };
        const NOT_INITIALIZED: Pubkey = pubkey!("3W6YcoQyFcrSo6K9vixhM2Cfvtjv4KeKSH1FaEKJF1Ug");
        const INITIALIZED: Pubkey = pubkey!("FNK9gB5E3cntDRiy3LHwtwQC6qhbVgdBLBMqjRZLEYiK");
        const EXPECTED_TABLE: [Pubkey; 3] = [
            pubkey!("So11111111111111111111111111111111111111112"),
            pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),
            pubkey!("8m3uKEn4fMPNVr7nv6RmQYktT4zRqEZzhuZDpG8hQZT4"),
        ];
        const DEST: Pubkey = pubkey!("8X35rQUK2u9hfn8rMPwwr6ZSEUhbmfDPEapp589XyoM1");
        const TEST_LOOKUP_TABLE_ADDRESS: Pubkey =
            pubkey!("njdSrqZgR1gZhLvGoX6wzhSioAczdN669SVt3nktiJe");
        const TEST_LOOKUP_TABLE_STATE: [Pubkey; 6] = [
            pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
            pubkey!("11111111111111111111111111111111"),
            pubkey!("ComputeBudget111111111111111111111111111111"),
            pubkey!("AddressLookupTab1e1111111111111111111111111"),
            pubkey!("8X35rQUK2u9hfn8rMPwwr6ZSEUhbmfDPEapp589XyoM1"),
            pubkey!("215r9xfTFVYcE9g3fAUGowauM84egyUvFCbSo3LKNaep"),
        ];
        #[tokio::test]
        async fn test_lookup_table() -> anyhow::Result<()> {
            let (_, rpc) = init()?;

            let result = fetch_lookup_tables(&[NOT_INITIALIZED], &rpc).await?;
            assert!(result.is_empty());

            let result = fetch_lookup_tables(&[INITIALIZED], &rpc).await?;
            assert_eq!(1, result.len());
            assert_eq!(result[0].key, INITIALIZED);
            assert_eq!(result[0].addresses, EXPECTED_TABLE);

            let result = fetch_lookup_tables(&[NOT_INITIALIZED, INITIALIZED], &rpc).await?;
            assert_eq!(1, result.len());
            assert_eq!(result[0].key, INITIALIZED);
            assert_eq!(result[0].addresses, EXPECTED_TABLE);

            let result = fetch_lookup_tables(&[], &rpc).await?;
            assert!(result.is_empty());
            Ok(())
        }

        fn random_instructions(payer: &Pubkey) -> Vec<solana_instruction::Instruction> {
            vec![
                solana_system_interface::instruction::transfer(payer, &DEST, 1),
                solana_system_interface::instruction::transfer(payer, &DEST, 2),
                solana_system_interface::instruction::transfer(payer, &DEST, 3),
            ]
        }

        #[tokio::test]
        async fn test_lookup_table_tx() -> anyhow::Result<()> {
            let (kp, rpc) = init()?;
            let pkg = "github.com/carteraMesh/nitrogen";
            let op = "lookup_tables_keys";
            let payer = kp.pubkey();
            let sig = TransactionBuilder::builder()
                .instructions(random_instructions(&payer))
                .lookup_tables_keys(vec![TEST_LOOKUP_TABLE_ADDRESS])
                .build()
                .with_memo(op, &[&payer])
                .with_memo(pkg, &[&payer])
                .send(&rpc, &payer, &[&kp])
                .await?;
            info!("builder {op} {sig}");
            let op = "address_lookup_table";
            let sig = TransactionBuilder::builder()
                .instructions(random_instructions(&payer))
                .address_lookup_tables(vec![AddressLookupTableAccount {
                    key: TEST_LOOKUP_TABLE_ADDRESS,
                    addresses: TEST_LOOKUP_TABLE_STATE.to_vec(),
                }])
                .build()
                .with_memo(op, &[&payer])
                .with_memo(pkg, &[&payer])
                .send(&rpc, &payer, &[&kp])
                .await?;

            info!("builder {op} {sig}");

            let op = "with_lookup_keys";
            let sig = TransactionBuilder::builder()
                .instructions(random_instructions(&payer))
                .build()
                .with_lookup_keys(vec![TEST_LOOKUP_TABLE_ADDRESS])
                .with_memo(op, &[&payer])
                .with_memo(pkg, &[&payer])
                .send(&rpc, &payer, &[&kp])
                .await?;
            info!("builder {op} {sig}");

            let op = "with_address_tables";
            let sig = TransactionBuilder::builder()
                .instructions(random_instructions(&payer))
                .build()
                .with_address_tables(vec![AddressLookupTableAccount {
                    key: TEST_LOOKUP_TABLE_ADDRESS,
                    addresses: TEST_LOOKUP_TABLE_STATE.to_vec(),
                }])
                .with_memo(op, &[&payer])
                .with_memo(pkg, &[&payer])
                .send(&rpc, &payer, &[&kp])
                .await?;
            info!("builder {op} {sig}");

            Ok(())
        }
    }
}
