#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
use {
    super::{Error, InstructionBuilder, IntoInstruction, Result},
    base64::prelude::*,
    borsh::BorshSerialize,
    solana_hash::Hash,
    solana_instruction::Instruction,
    solana_message::{AddressLookupTableAccount, VersionedMessage, v0::Message},
    solana_pubkey::Pubkey,
    solana_rpc_client_api::config::RpcSimulateTransactionConfig,
    solana_signature::Signature,
    solana_signer::signers::Signers,
    solana_transaction::versioned::VersionedTransaction,
    std::fmt::Debug,
    tracing::debug,
};

/// Builder/Helper for creating and sending Solana [`VersionedTransaction`]s,
/// with [`AddressLookupTableAccount`] support
///
/// See [`VersionedTransaction`] and [`Message`] for official reference
#[derive(bon::Builder, Clone, Default)]
pub struct TransactionBuilder {
    pub instructions: Vec<Instruction>,
    /// Pukeys that resolve to [`AddressLookupTableAccount`] via
    /// [`crate::lookup::fetch_lookup_tables`]
    pub lookup_tables_keys: Option<Vec<Pubkey>>,

    /// For [`VersionedTransaction`]
    pub address_lookup_tables: Option<Vec<AddressLookupTableAccount>>,
}

impl Debug for TransactionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#inxs={}", self.instructions.len())
    }
}

#[cfg(feature = "blocking")]
impl TransactionBuilder {}

#[cfg(not(feature = "blocking"))]
impl TransactionBuilder {
    async fn get_latest_blockhash(rpc: &RpcClient) -> Result<Hash> {
        rpc.get_latest_blockhash()
            .await
            .map_err(|e| Error::SolanaRpcError(format!("failed to get latest blockhash: {e}")))
    }

    pub async fn create_message(
        &self,
        payer: &Pubkey,
        rpc: &RpcClient,
    ) -> Result<VersionedMessage> {
        Ok(match &self.address_lookup_tables {
            Some(accounts) => VersionedMessage::V0(Message::try_compile(
                payer,
                self.instructions.as_ref(),
                accounts,
                TransactionBuilder::get_latest_blockhash(rpc).await?,
            )?),
            None => match self.lookup_tables_keys {
                Some(ref keys) => {
                    let accounts = super::lookup::fetch_lookup_tables(keys, rpc).await?;
                    VersionedMessage::V0(Message::try_compile(
                        payer,
                        self.instructions.as_ref(),
                        &accounts,
                        TransactionBuilder::get_latest_blockhash(rpc).await?,
                    )?)
                }
                None => VersionedMessage::Legacy(solana_message::Message::new_with_blockhash(
                    &self.instructions,
                    Some(payer),
                    &TransactionBuilder::get_latest_blockhash(rpc).await?,
                )),
            },
        })
    }

    /// Simulates the [`VersionedTransaction`] using
    /// [`RpcClient::simulate_transaction_with_config`].
    pub async fn simulate<S: Signers + ?Sized>(
        &self,
        payer: &Pubkey,
        signers: &S,
        rpc: &RpcClient,
        config: RpcSimulateTransactionConfig,
    ) -> Result<()> {
        let tx = VersionedTransaction::try_new(self.create_message(payer, rpc).await?, signers)?;
        self.simulate_internal(rpc, &tx, config).await
    }

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
    #[tracing::instrument(skip(self, rpc, signers), level = tracing::Level::INFO)]
    pub async fn send<S: Signers + ?Sized>(
        &self,
        rpc: &RpcClient,
        payer: &Pubkey,
        signers: &S,
    ) -> Result<Signature> {
        let tx = VersionedTransaction::try_new(self.create_message(payer, rpc).await?, signers)?;
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

impl TransactionBuilder {
    /// When [`TransactionBuilder::send`] or [`TransactionBuilder::simulate`] is
    /// called, these keys will be used via RPC and be converted into
    /// [`AddressLookupTableAccount`].
    pub fn with_lookup_keys<I, P>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<Pubkey>,
    {
        let new_keys: Vec<Pubkey> = keys.into_iter().map(|k| k.into()).collect();
        match self.lookup_tables_keys {
            Some(ref mut existing) => existing.extend(new_keys),
            None => self.lookup_tables_keys = Some(new_keys),
        }
        self
    }

    /// This function takes precedence over
    /// [`TransactionBuilder::with_lookup_keys`]
    ///
    ///
    /// When [`TransactionBuilder::send`] or [`TransactionBuilder::simulate`] is
    /// called, and will be used via RPC and be converted into
    /// [`AddressLookupTableAccount`].
    pub fn with_address_tables<I, P>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<AddressLookupTableAccount>,
    {
        let new_tables: Vec<AddressLookupTableAccount> =
            keys.into_iter().map(|k| k.into()).collect();
        match self.address_lookup_tables {
            Some(ref mut existing) => existing.extend(new_tables),
            None => self.address_lookup_tables = Some(new_tables),
        }
        self
    }

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
}

impl From<TransactionBuilder> for Vec<Instruction> {
    fn from(builder: TransactionBuilder) -> Self {
        builder.instructions
    }
}

impl From<Vec<Instruction>> for TransactionBuilder {
    fn from(instructions: Vec<Instruction>) -> Self {
        TransactionBuilder::builder()
            .instructions(instructions)
            .build()
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

    #[test]
    fn test_with_lookup_keys_extending() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let pk3 = Pubkey::new_unique();
        let pk4 = Pubkey::new_unique();

        let tx = TransactionBuilder::default()
            .with_lookup_keys([pk1, pk2])
            .with_lookup_keys(vec![pk3, pk4]);

        assert_eq!(tx.lookup_tables_keys.as_ref().unwrap().len(), 4);
        assert_eq!(tx.lookup_tables_keys.unwrap(), vec![pk1, pk2, pk3, pk4]);
    }

    #[test]
    fn test_with_address_tables_extending() {
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();

        let table1 = AddressLookupTableAccount {
            key: pk1,
            addresses: vec![],
        };
        let table2 = AddressLookupTableAccount {
            key: pk2,
            addresses: vec![],
        };

        let tx = TransactionBuilder::default()
            .with_address_tables([table1.clone()])
            .with_address_tables(vec![table2.clone()]);

        let tables = tx.address_lookup_tables.unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].key, pk1);
        assert_eq!(tables[1].key, pk2);
    }
}
