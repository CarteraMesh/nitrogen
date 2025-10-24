use {
    clap::{Args, Parser, Subcommand},
    std::fmt::Debug,
};

#[derive(Parser)]
#[command(name = "circle-cctp")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct BurnArgs {
    #[arg(long, default_value = "10")]
    pub amount: u64,
    #[arg(long, default_value = "6")]
    pub destination_chain: u32,
    #[arg(long, help = "default is to EVM")]
    pub to_sol: bool,
    pub destination: String,
}

impl Debug for BurnArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[amount={},dest={},chain={}]",
            self.amount, self.destination, self.destination_chain
        )
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Burn(BurnArgs),
    Reclaim,
    Recv { tx_hash: String },
}
