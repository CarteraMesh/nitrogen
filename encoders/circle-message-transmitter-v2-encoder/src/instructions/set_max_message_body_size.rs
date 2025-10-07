use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SetMaxMessageBodySize {
    pub params: SetMaxMessageBodySizeParams,
}

impl borsh::BorshSerialize for SetMaxMessageBodySize {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[168, 178, 8, 117, 217, 167, 219, 31])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SetMaxMessageBodySize {
    pub fn build(
        &self,
        owner: Pubkey,
        message_transmitter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(4);
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
