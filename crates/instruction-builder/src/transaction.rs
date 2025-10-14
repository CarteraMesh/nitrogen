#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
use {
    super::{Error, InstructionBuilder, IntoInstruction, Result},
    base64::prelude::*,
    borsh::BorshSerialize,
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
    solana_rpc_client_api::config::RpcSimulateTransactionConfig,
    solana_signature::Signature,
    solana_signer::signers::Signers,
    solana_transaction::Transaction,
    tracing::debug,
};

#[derive(bon::Builder, Debug)]
pub struct TransactionBuilder {
    pub instructions: Vec<Instruction>,
}

impl TransactionBuilder {
    pub fn push<T: IntoInstruction>(mut self, builder: T) -> Self {
        self.instructions.push(builder.into_instruction());
        self
    }

    pub fn append<T: BorshSerialize>(mut self, builders: Vec<InstructionBuilder<T>>) -> Self {
        self.instructions
            .extend(builders.into_iter().map(|b| b.instruction()));
        self
    }

    #[cfg(feature = "blocking")]
    pub fn send<S: Signers + ?Sized>(
        &self,
        rpc: &RpcClient,
        payer: Option<&Pubkey>,
        signers: &S,
    ) -> Result<Signature> {
        let recent_blockhash = rpc
            .get_latest_blockhash()
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))?;
        let tx = Transaction::new_signed_with_payer(
            &self.instructions,
            payer,
            signers,
            recent_blockhash,
        );
        let transaction_base64 = BASE64_STANDARD.encode(bincode::serialize(&tx)?);
        debug!("{transaction_base64}");
        let result = rpc
            .simulate_transaction_with_config(&tx, RpcSimulateTransactionConfig {
                sig_verify: true,
                ..RpcSimulateTransactionConfig::default()
            })
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))?;
        if let Some(e) = result.value.err {
            let logs = result.value.logs.unwrap_or(Vec::new());
            let msg = format!("{e}\nbase64: {transaction_base64}\n{}", logs.join("\n"));
            return Err(Error::SolanaSimulateFailure(msg));
        }
        rpc.send_and_confirm_transaction(&tx)
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))
    }

    #[cfg(not(feature = "blocking"))]
    pub async fn send<S: Signers + ?Sized>(
        &self,
        rpc: &RpcClient,
        payer: Option<&Pubkey>,
        signers: &S,
    ) -> Result<Signature> {
        let recent_blockhash = rpc
            .get_latest_blockhash()
            .await
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))?;
        let tx = Transaction::new_signed_with_payer(
            &self.instructions,
            payer,
            signers,
            recent_blockhash,
        );
        let transaction_base64 = BASE64_STANDARD.encode(bincode::serialize(&tx)?);
        debug!("{transaction_base64}");
        let result = rpc
            .simulate_transaction_with_config(&tx, RpcSimulateTransactionConfig {
                sig_verify: true,
                ..RpcSimulateTransactionConfig::default()
            })
            .await
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))?;

        if let Some(e) = result.value.err {
            let logs = result.value.logs.unwrap_or(Vec::new());
            let msg = format!("{e}\nbase64: {transaction_base64}\n{}", logs.join("\n"));
            return Err(Error::SolanaSimulateFailure(msg));
        }
        rpc.send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| crate::Error::SolanRpcError(e.to_string()))
    }
}

impl IntoIterator for TransactionBuilder {
    type IntoIter = std::vec::IntoIter<Instruction>;
    type Item = Instruction;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}
