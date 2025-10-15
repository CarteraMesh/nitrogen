use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error("Failed simulation: {0}")]
    SolanaSimulateFailure(String),

    #[error("Failed RPC call: {0}")]
    SolanaRpcError(String),

    #[error(transparent)]
    BorshError(#[from] std::io::Error),
}
