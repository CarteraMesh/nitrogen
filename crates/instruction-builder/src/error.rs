use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error("No instructions provided")]
    NoInstructions,

    #[error("Failed simulation: {0}")]
    SolanaSimulateFailure(String),

    #[error("Failed RPC call: {0}")]
    SolanaRpcError(String),

    #[error(transparent)]
    BorshError(#[from] std::io::Error),

    #[error(transparent)]
    ParseAccountError(#[from] solana_account_decoder::parse_account_data::ParseAccountError),

    #[error(transparent)]
    ParsePubkeyError(#[from] solana_pubkey::ParsePubkeyError),

    #[error(transparent)]
    MessageError(#[from] solana_message::CompileError),

    #[error(transparent)]
    SignerError(#[from] solana_signer::SignerError),

    #[error(transparent)]
    NumConversionError(#[from] std::num::TryFromIntError),

    #[error("Invalid compute units {0} {1}")]
    InvalidComputeUnits(u64, String),
}
