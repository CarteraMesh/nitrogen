use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Pause {
    pub params: PauseParams,
}

impl borsh::BorshSerialize for Pause {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[211, 22, 221, 251, 74, 121, 193, 47])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl Pause {
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
