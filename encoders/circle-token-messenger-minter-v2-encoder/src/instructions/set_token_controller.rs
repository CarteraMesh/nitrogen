use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SetTokenController {
    pub params: SetTokenControllerParams,
}

impl borsh::BorshSerialize for SetTokenController {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[88, 6, 98, 10, 79, 59, 15, 24])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SetTokenController {
    pub fn build(
        &self,
        owner: Pubkey,
        token_messenger: Pubkey,
        token_minter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(5);
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(AccountMeta::new(token_minter, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
