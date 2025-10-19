#![doc = include_str!("../README.md")]

use {
    borsh::BorshSerialize,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

mod instruction;
pub use instruction::*;

/// Derives a PDA and returns an [`AccountMeta`].
///
/// Wraps [`Pubkey::find_program_address`] and creates an [`AccountMeta`] with
/// `is_signer: false`.
pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey, read_only: bool) -> AccountMeta {
    if read_only {
        AccountMeta::new_readonly(Pubkey::find_program_address(seeds, program_id).0, false)
    } else {
        AccountMeta::new(Pubkey::find_program_address(seeds, program_id).0, false)
    }
}

/// Converts types into [`Instruction`].
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

#[cfg(test)]
mod tests {
    use {
        super::InstructionBuilder,
        borsh::BorshSerialize,
        solana_instruction::AccountMeta,
        solana_pubkey::Pubkey,
    };

    #[derive(BorshSerialize)]
    struct MemoData {
        pub memo: Vec<u8>,
    }

    impl From<&str> for MemoData {
        fn from(value: &str) -> Self {
            MemoData {
                memo: value.to_string().into_bytes(),
            }
        }
    }

    #[test]
    fn test_remaining_accounts() {
        let memo: MemoData = "With remaining accounts".into();
        let base_accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];
        let remaining_accounts = vec![
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        ];

        let instruction_builder = InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(base_accounts.clone())
            .params(memo)
            .build()
            .remaining_accounts(remaining_accounts.clone());

        let instruction = instruction_builder.instruction();

        // Verify the instruction has all accounts (base + remaining)
        assert_eq!(
            instruction.accounts.len(),
            base_accounts.len() + remaining_accounts.len()
        );
        assert_eq!(instruction.program_id, spl_memo::id());
    }

    #[test]
    fn test_instruction_creation() {
        let memo: MemoData = "Test instruction creation".into();
        let accounts = vec![AccountMeta::new_readonly(Pubkey::new_unique(), true)];

        let builder = InstructionBuilder::builder()
            .program_id(spl_memo::id())
            .accounts(accounts.clone())
            .params(memo)
            .build();

        let instruction = builder.instruction();
        assert_eq!(instruction.program_id, spl_memo::id());
        assert_eq!(instruction.accounts.len(), accounts.len());
    }
}
