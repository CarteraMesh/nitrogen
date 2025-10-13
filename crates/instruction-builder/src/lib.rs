#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
use {
    base64::prelude::*,
    borsh::BorshSerialize,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_signature::Signature,
    solana_signer::signers::Signers,
    solana_transaction::Transaction,
    tracing::debug,
};

mod error;
pub use error::*;
pub type Result<T> = std::result::Result<T, Error>;

pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey, read_only: bool) -> AccountMeta {
    if read_only {
        AccountMeta::new_readonly(Pubkey::find_program_address(seeds, program_id).0, false)
    } else {
        AccountMeta::new(Pubkey::find_program_address(seeds, program_id).0, false)
    }
}

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
        use solana_rpc_client_api::config::RpcSimulateTransactionConfig;

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
        use solana_rpc_client_api::config::RpcSimulateTransactionConfig;

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

#[derive(bon::Builder, Debug)]
pub struct InstructionBuilder<T: BorshSerialize> {
    pub params: T,
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
}

impl<T: BorshSerialize> InstructionBuilder<T> {
    pub fn remaining_accounts(mut self, mut account: Vec<AccountMeta>) -> Self {
        self.accounts.append(&mut account);
        self
    }

    pub fn tx(self) -> TransactionBuilder {
        TransactionBuilder {
            instructions: vec![Instruction::new_with_borsh(
                self.program_id,
                &self.params,
                self.accounts,
            )],
        }
    }

    pub fn instruction(self) -> Instruction {
        Instruction::new_with_borsh(self.program_id, &self.params, self.accounts)
    }
}

impl<T: BorshSerialize> From<InstructionBuilder<T>> for Instruction {
    fn from(builder: InstructionBuilder<T>) -> Self {
        builder.instruction()
    }
}

impl<T: BorshSerialize> From<InstructionBuilder<T>> for TransactionBuilder {
    fn from(builder: InstructionBuilder<T>) -> Self {
        builder.tx()
    }
}

impl<T: BorshSerialize> Extend<InstructionBuilder<T>> for TransactionBuilder {
    fn extend<I: IntoIterator<Item = InstructionBuilder<T>>>(&mut self, iter: I) {
        self.instructions
            .extend(iter.into_iter().map(|b| b.instruction()));
    }
}

impl IntoIterator for TransactionBuilder {
    type IntoIter = std::vec::IntoIter<Instruction>;
    type Item = Instruction;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

pub trait IntoInstruction {
    fn into_instruction(self) -> Instruction;
}

impl<T: BorshSerialize> IntoInstruction for InstructionBuilder<T> {
    fn into_instruction(self) -> Instruction {
        self.instruction()
    }
}

impl IntoInstruction for Instruction {
    fn into_instruction(self) -> Instruction {
        self
    }
}
