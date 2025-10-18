#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
#[cfg(not(feature = "blocking"))]
use solana_rpc_client_api::response::RpcSimulateTransactionResult;
use {
    super::{Error, InstructionBuilder, IntoInstruction, Result},
    base64::prelude::*,
    borsh::BorshSerialize,
    solana_compute_budget_interface::ComputeBudgetInstruction,
    solana_hash::Hash,
    solana_instruction::Instruction,
    solana_message::{AddressLookupTableAccount, VersionedMessage, v0::Message},
    solana_pubkey::Pubkey,
    solana_rpc_client_api::{config::RpcSimulateTransactionConfig, response::RpcPrioritizationFee},
    solana_signature::Signature,
    solana_signer::signers::Signers,
    solana_transaction::versioned::VersionedTransaction,
    std::fmt::Debug,
    tracing::debug,
};

const SOLANA_MAX_COMPUTE_UNITS: u32 = 1_400_000;

/// Builder/Helper for creating and sending Solana [`VersionedTransaction`]s,
/// with [`AddressLookupTableAccount`] support
///
/// See [`VersionedTransaction`] and [`Message`] for official reference
#[derive(bon::Builder, Clone, Default)]
pub struct TransactionBuilder {
    pub instructions: Vec<Instruction>,
    /// [`Pubkey`]s that resolve to [`AddressLookupTableAccount`] via
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
    ) -> Result<RpcSimulateTransactionResult> {
        let tx = VersionedTransaction::try_new(self.create_message(payer, rpc).await?, signers)?;
        self.simulate_internal(rpc, &tx, config).await
    }

    async fn simulate_internal(
        &self,
        rpc: &RpcClient,
        tx: &VersionedTransaction,
        config: RpcSimulateTransactionConfig,
    ) -> Result<RpcSimulateTransactionResult> {
        let transaction_base64 = BASE64_STANDARD.encode(bincode::serialize(&tx)?);
        debug!("BASE64 tx: {transaction_base64}");
        let result = rpc
            .simulate_transaction_with_config(tx, config)
            .await
            .map_err(|e| Error::SolanaRpcError(format!("failed to simulate transaction: {e}")))?;

        if let Some(e) = result.value.err {
            let logs = result.value.logs.unwrap_or(Vec::new());
            let msg = format!("{e}\nbase64: {transaction_base64}\n{}", logs.join("\n"));
            return Err(Error::SolanaSimulateFailure(msg));
        }
        Ok(result.value)
    }

    /// Simulates, signs, and sends the transaction using
    /// [`RpcClient::send_and_confirm_transaction`].
    #[tracing::instrument(skip(rpc, signers), level = tracing::Level::INFO)]
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

    pub async fn get_recent_prioritization_fees(
        rpc: &RpcClient,
        accounts: &[Pubkey],
    ) -> Result<Vec<RpcPrioritizationFee>> {
        rpc.get_recent_prioritization_fees(accounts)
            .await
            .map_err(|e| {
                Error::SolanaRpcError(format!("failed to get_recent_prioritization_fees: {e}"))
            })
    }

    /// Quick and dirty fee estimation using recent prioritization fees.
    ///
    /// This convenience method fetches recent prioritization fees and
    /// automatically adds ComputeBudget instructions to the beginning of
    /// your transaction.
    ///
    /// **NOTE** use a real RPC Fee Service if you want more accurate fee
    /// estimation.  This method is for convenience and may not be suitable
    /// for all use cases.
    ///
    /// # Arguments
    /// * `rpc` - RPC client for fetching recent fees
    /// * `max_prioritization_fee` - Optional cap on prioritization fee
    ///   (microlamports per CU)
    /// * `accounts` - Write-locked account addresses to query for relevant
    ///   prioritization fees. Fees are filtered to transactions that interact
    ///   with these accounts. Use program IDs and frequently-accessed accounts
    ///   for best results.
    /// * `percentile` - Fee percentile to use (default: 75th percentile)
    ///
    /// # Example
    /// ```no_run
    /// # use nitrogen_instruction_builder::TransactionBuilder;
    /// # use solana_pubkey::Pubkey;
    /// # async fn example(builder: TransactionBuilder, payer: Pubkey, rpc: solana_rpc_client::nonblocking::rpc_client::RpcClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tx = builder
    ///     .with_priority_fees(
    ///         &payer,
    ///         &rpc,
    ///         Some(5_000_000), // Cap at 5M microlamports/CU
    ///         &[solana_system_interface::program::ID],
    ///         Some(50), // Use 50th percentile (median)
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Special Considerations
    /// If you use priority fees with a Durable Nonce Transaction, you must
    /// ensure the AdvanceNonce instruction is your transaction's first
    /// instruction. This is critical to ensure your transaction is
    /// successful; otherwise, it will fail.
    ///
    ///
    ///
    /// Reference: <https://solana.com/developers/guides/advanced/how-to-use-priority-fees>
    #[tracing::instrument(skip(rpc, payer, accounts), level = tracing::Level::DEBUG)]
    pub async fn with_priority_fees(
        self,
        payer: &Pubkey,
        rpc: &RpcClient,
        accounts: &[Pubkey],
        max_prioritization_fee: Option<u64>,
        percentile: Option<u8>,
    ) -> Result<Self> {
        if self.instructions.is_empty() {
            return Err(crate::Error::NoInstructions);
        }
        if self
            .instructions
            .iter()
            .any(|ix| ix.program_id == solana_compute_budget_interface::ID)
        {
            tracing::warn!("ComputeBudgetProgram already exists");
            return Ok(self);
        }
        let fees = TransactionBuilder::get_recent_prioritization_fees(rpc, accounts).await?;
        let tx = self.unsigned_tx(payer, rpc).await?;
        let sim_result = self
            .simulate_internal(rpc, &tx, RpcSimulateTransactionConfig {
                sig_verify: false,
                ..Default::default()
            })
            .await?;

        let units = sim_result
            .units_consumed
            .ok_or(crate::Error::InvalidComputeUnits(
                0,
                "RPC returned invalid units".to_string(),
            ))?;
        Ok(self.calc_fees(fees, units.try_into()?, max_prioritization_fee, percentile))
    }

    pub async fn unsigned_tx(
        &self,
        payer: &Pubkey,
        rpc: &RpcClient,
    ) -> Result<VersionedTransaction> {
        let message = self.create_message(payer, rpc).await?;
        let num_sigs = message.header().num_required_signatures as usize;
        Ok(VersionedTransaction {
            signatures: vec![Signature::default(); num_sigs],
            message,
        })
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

    /// Add ComputeBudget instructions to beginning of the transaction. Fails if
    /// ComputeBudget instructions are already present.
    ///
    ///
    /// Use [`TransactionBuilder::unsigned_tx`] to get a transaction for
    /// fee simulation.
    pub fn prepend_compute_budget_instructions(
        mut self,
        units: u32,
        priority_fees: u64,
    ) -> Result<Self> {
        if self
            .instructions
            .iter()
            .any(|ix| ix.program_id == solana_compute_budget_interface::ID)
        {
            return Err(crate::Error::InvalidComputeUnits(
                units.into(),
                "computes is about max solana compute units".to_owned(),
            ));
        }

        self.instructions.splice(0..0, vec![
            ComputeBudgetInstruction::set_compute_unit_limit(units),
            ComputeBudgetInstruction::set_compute_unit_price(priority_fees),
        ]);
        Ok(self)
    }

    fn calc_fees(
        mut self,
        fees: Vec<RpcPrioritizationFee>,
        compute_unit_limit: u32,
        max_prioritization_fee: Option<u64>,
        percentile: Option<u8>,
    ) -> Self {
        if fees.is_empty() {
            tracing::warn!("no RpcPrioritizationFee to calculate fees");
            return self;
        }

        let percentile = percentile.unwrap_or(75).min(100);
        let mut sorted_fees: Vec<u64> = fees.iter().map(|f| f.prioritization_fee).collect();
        sorted_fees.sort();

        let index = (sorted_fees.len() * percentile as usize).saturating_sub(1) / 100;
        let priority_fee = max_prioritization_fee
            .map(|max| sorted_fees[index].min(max))
            .unwrap_or(sorted_fees[index]);

        // Add buffer but cap at Solana's maximum
        let buffered_limit = compute_unit_limit
            .saturating_add(compute_unit_limit / 10)
            .min(SOLANA_MAX_COMPUTE_UNITS);
        // Prepend compute budget instructions to the main instructions
        let compute_budget_instructions = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(buffered_limit),
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee),
        ];

        self.instructions.splice(0..0, compute_budget_instructions);
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
