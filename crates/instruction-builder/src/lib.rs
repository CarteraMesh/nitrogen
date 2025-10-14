use {
    borsh::BorshSerialize,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

mod error;
mod instruction;
mod transaction;
pub use {error::*, instruction::*, transaction::*};
pub type Result<T> = std::result::Result<T, Error>;

pub fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey, read_only: bool) -> AccountMeta {
    if read_only {
        AccountMeta::new_readonly(Pubkey::find_program_address(seeds, program_id).0, false)
    } else {
        AccountMeta::new(Pubkey::find_program_address(seeds, program_id).0, false)
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
