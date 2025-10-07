use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct UndenylistAccount {
    pub params: UndenylistParams,
}

impl borsh::BorshSerialize for UndenylistAccount {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[57, 36, 43, 168, 62, 172, 33, 39])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl UndenylistAccount {
    pub fn build(
        &self,
        payer: Pubkey,
        denylister: Pubkey,
        token_messenger: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(7);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(denylister, true));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(crate::derive_pda(
            &[b"denylist_account", self.params.account.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("11111111111111111111111111111111"),
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
