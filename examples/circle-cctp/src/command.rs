use {
    clap::{Parser, Subcommand},
    solana_pubkey::Pubkey,
};

#[derive(Parser)]
#[command(name = "circle-cctp")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Bridge {
        #[arg(long, default_value = "1")]
        amount: u64,
        #[arg(long, default_value = "6")]
        destination_chain: u32,
        destination: String,
    },
    Reclaim {
        message_sent_event_account: Pubkey,
        tx_hash: String,
    },
}
