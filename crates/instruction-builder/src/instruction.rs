use {
    super::TransactionBuilder,
    borsh::BorshSerialize,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

/// Builder for creating Solana [`Instruction`]s with Borsh-serialized data.
#[derive(bon::Builder, Debug, Clone)]
pub struct InstructionBuilder<T: BorshSerialize> {
    pub params: T,
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
}

impl<T: BorshSerialize> InstructionBuilder<T> {
    /// Appends additional accounts to the instruction.
    pub fn remaining_accounts(mut self, mut account: Vec<AccountMeta>) -> Self {
        self.accounts.append(&mut account);
        self
    }

    /// Converts to a [`TransactionBuilder`] with this instruction.
    pub fn tx(self) -> TransactionBuilder {
        TransactionBuilder::builder()
            .instructions(vec![Instruction::new_with_borsh(
                self.program_id,
                &self.params,
                self.accounts,
            )])
            .build()
    }

    /// Builds the [`Instruction`] using [`Instruction::new_with_borsh`].
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
