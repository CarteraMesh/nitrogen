#[cfg(feature = "blocking")]
mod tests {
    use {
        borsh::BorshSerialize,
        nitrogen_instruction_builder::InstructionBuilder,
        solana_commitment_config::CommitmentConfig,
        solana_instruction::AccountMeta,
        solana_keypair::Keypair,
        solana_rpc_client::rpc_client::RpcClient,
        solana_signer::Signer,
        std::{env, sync::Once},
        tracing::info,
        tracing_subscriber::{EnvFilter, fmt::format::FmtSpan},
    };

    pub static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            if env::var("CI").is_err() {
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

    #[derive(BorshSerialize)]
    struct MemoData {
        pub memo: Vec<u8>,
    }

    impl From<&str> for MemoData {
        fn from(s: &str) -> Self {
            MemoData {
                memo: s.to_string().into_bytes(),
            }
        }
    }

    #[allow(clippy::expect_fun_call)]
    fn init() -> anyhow::Result<(Keypair, RpcClient)> {
        setup();
        let kp_file = env::var("KEYPAIR_FILE").ok();
        let owner = if let Some(kp) = kp_file {
            solana_keypair::read_keypair_file(&kp)
                .expect(&format!("unable to load keypair file {kp}"))
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

    #[test]
    fn test_blocking_send() -> anyhow::Result<()> {
        let (kp, rpc) = init()?;
        let memo: MemoData = "Blocking test".into();
        let accounts = vec![AccountMeta::new_readonly(kp.pubkey(), true)];

        let tx = InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts)
            .params(memo)
            .build()
            .tx();

        let sig = tx.send(&rpc, Some(&kp.pubkey()), &[&kp])?;
        info!("Blocking tx: {sig}");
        Ok(())
    }
}
