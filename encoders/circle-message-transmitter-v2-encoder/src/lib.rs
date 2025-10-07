use {
    solana_instruction::AccountMeta,
    solana_pubkey::{Pubkey, declare_id},
};
pub struct MessageTransmitterV2Encoder;
pub mod accounts;
pub mod instructions;
pub mod types;

declare_id!("CCTPV2Sm4AdWt5296sk4P66VBZ7bEhcARwFaaS9YPbeC");

pub(crate) fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey, read_only: bool) -> AccountMeta {
    if read_only {
        AccountMeta::new_readonly(Pubkey::find_program_address(seeds, program_id).0, false)
    } else {
        AccountMeta::new(Pubkey::find_program_address(seeds, program_id).0, false)
    }
}
