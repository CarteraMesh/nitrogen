#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
#[cfg(not(feature = "blocking"))]
use solana_transaction::versioned::VersionedTransaction;
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

/// Builder for creating and sending Solana [`Transaction`]s.
///
/// See [`RpcClient`] for underlying RPC methods.
#[derive(bon::Builder, Debug, Clone, Default)]
pub struct TransactionBuilder {
    pub instructions: Vec<Instruction>,
}

impl TransactionBuilder {
    pub fn with_memo(mut self, memo: impl AsRef<[u8]>, signer_pubkeys: &[&Pubkey]) -> Self {
        self.instructions
            .push(spl_memo::build_memo(memo.as_ref(), signer_pubkeys));
        self
    }

    /// Adds an instruction to the transaction.
    pub fn push<T: IntoInstruction>(mut self, builder: T) -> Self {
        self.instructions.push(builder.into_instruction());
        self
    }

    /// Appends multiple instructions to the transaction.
    pub fn append<T: BorshSerialize>(mut self, builders: Vec<InstructionBuilder<T>>) -> Self {
        self.instructions
            .extend(builders.into_iter().map(|b| b.instruction()));
        self
    }

    /// Simulates, signs, and sends the transaction using
    /// [`RpcClient::send_and_confirm_transaction`].
    #[cfg(feature = "blocking")]
    pub fn send<S: Signers + ?Sized>(
        &self,
        rpc: &RpcClient,
        payer: Option<&Pubkey>,
        signers: &S,
    ) -> Result<Signature> {
        let recent_blockhash = rpc
            .get_latest_blockhash()
            .map_err(|e| Error::SolanaRpcError(format!("failed to get latest blockhash: {e}")))?;
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
            .map_err(|e| Error::SolanaRpcError(format!("failed to simulate: {e}")))?;
        if let Some(e) = result.value.err {
            let logs = result.value.logs.unwrap_or(Vec::new());
            let msg = format!("{e}\nbase64: {transaction_base64}\n{}", logs.join("\n"));
            return Err(Error::SolanaSimulateFailure(msg));
        }
        rpc.send_and_confirm_transaction(&tx)
            .map_err(|e| Error::SolanaRpcError(format!("failed to send transaction: {e}")))
    }

    /// Simulates the transaction using
    /// [`RpcClient::simulate_transaction_with_config`].
    #[cfg(not(feature = "blocking"))]
    pub async fn simulate<S: Signers + ?Sized>(
        &self,
        payer: Option<&Pubkey>,
        signers: &S,
        rpc: &RpcClient,
        config: RpcSimulateTransactionConfig,
    ) -> Result<()> {
        let recent_blockhash = rpc
            .get_latest_blockhash()
            .await
            .map_err(|e| Error::SolanaRpcError(format!("failed to get latest blockhash: {e}")))?;

        let tx: VersionedTransaction = Transaction::new_signed_with_payer(
            &self.instructions,
            payer,
            signers,
            recent_blockhash,
        )
        .into();
        self.simulate_internal(rpc, &tx, config).await
    }

    #[cfg(not(feature = "blocking"))]
    async fn simulate_internal(
        &self,
        rpc: &RpcClient,
        tx: &VersionedTransaction,
        config: RpcSimulateTransactionConfig,
    ) -> Result<()> {
        let transaction_base64 = BASE64_STANDARD.encode(bincode::serialize(&tx)?);
        debug!("{transaction_base64}");
        let result = rpc
            .simulate_transaction_with_config(tx, config)
            .await
            .map_err(|e| Error::SolanaRpcError(format!("failed to simulate transaction: {e}")))?;

        if let Some(e) = result.value.err {
            let logs = result.value.logs.unwrap_or(Vec::new());
            let msg = format!("{e}\nbase64: {transaction_base64}\n{}", logs.join("\n"));
            return Err(Error::SolanaSimulateFailure(msg));
        }
        Ok(())
    }

    /// Simulates, signs, and sends the transaction using
    /// [`RpcClient::send_and_confirm_transaction`].
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
            .map_err(|e| Error::SolanaRpcError(format!("failed to get latest blockhash: {e}")))?;
        let tx: VersionedTransaction = Transaction::new_signed_with_payer(
            &self.instructions,
            payer,
            signers,
            recent_blockhash,
        )
        .into();
        self.simulate_internal(rpc, &tx, RpcSimulateTransactionConfig {
            sig_verify: true,
            ..Default::default()
        })
        .await?;
        rpc.send_and_confirm_transaction(&tx)
            .await
            .map_err(|e| Error::SolanaRpcError(format!("failed to send transaction: {e}")))
    }
}

impl Extend<Instruction> for TransactionBuilder {
    fn extend<I: IntoIterator<Item = Instruction>>(&mut self, iter: I) {
        self.instructions.extend(iter);
    }
}

impl IntoIterator for TransactionBuilder {
    type IntoIter = std::vec::IntoIter<Instruction>;
    type Item = Instruction;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_memo() {
        let tx = TransactionBuilder::default();
        let pk = spl_memo::id();
        let signer_pubkey = [&pk];
        let ref_msg = &[72, 101, 108, 108, 111];
        let tx = tx
            .with_memo("Hello world", &signer_pubkey)
            .with_memo(String::from("Hello"), &signer_pubkey)
            .with_memo(vec![72, 101, 108, 108, 111], &signer_pubkey)
            .with_memo(*ref_msg, &signer_pubkey)
            .with_memo([72, 101, 108, 108, 111], &signer_pubkey)
            .with_memo(b"Hello world", &signer_pubkey);

        assert_eq!(tx.instructions.len(), 6);
    }
}
