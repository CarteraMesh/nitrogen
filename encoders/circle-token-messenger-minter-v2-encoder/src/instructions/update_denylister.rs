use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct UpdateDenylister {
    pub params: UpdateDenylisterParams,
}

impl borsh::BorshSerialize for UpdateDenylister {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[193, 66, 198, 201, 84, 57, 14, 222])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl UpdateDenylister {
    pub fn build(
        &self,
        owner: Pubkey,
        token_messenger: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(4);
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new(token_messenger, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
