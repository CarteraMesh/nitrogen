use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "circle-cctp")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct BridgeArgs {
    #[arg(long, default_value = "10")]
    pub amount: u64,
    #[arg(long, default_value = "6")]
    pub destination_chain: u32,
    #[arg(long, help = "default is to EVM")]
    pub to_sol: bool,
    pub destination: String,
}

#[derive(Subcommand)]
pub enum Commands {
    Bridge(BridgeArgs),
    Reclaim,
    Recv { tx_hash: String },
}
