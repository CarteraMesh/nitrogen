//! Wrapper around Solana SDK types for building instructions and transactions.
//!
//! See [`solana_instruction`], [`solana_transaction`], and
//! [`solana_rpc_client`] for underlying types.
#![doc = include_str!("../README.md")]

use {
    borsh::BorshSerialize,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

mod error;
mod instruction;
mod lookup;
mod transaction;
pub use {error::*, instruction::*, lookup::*, transaction::*};
pub type Result<T> = std::result::Result<T, Error>;

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
