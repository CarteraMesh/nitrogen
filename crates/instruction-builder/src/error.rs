use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error("Failed simulation: {0}")]
    SolanaSimulateFailure(String),

    #[error(transparent)]
    SolanRpcError(#[from] solana_rpc_client_api::client_error::Error),
}
