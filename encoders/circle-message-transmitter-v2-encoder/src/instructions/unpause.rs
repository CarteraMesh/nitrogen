use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Unpause {
    pub params: UnpauseParams,
}

impl borsh::BorshSerialize for Unpause {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[169, 144, 4, 38, 10, 141, 188, 255])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl Unpause {
    pub fn build(
        &self,
        pauser: Pubkey,
        message_transmitter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(4);
        accounts.push(AccountMeta::new_readonly(pauser, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
