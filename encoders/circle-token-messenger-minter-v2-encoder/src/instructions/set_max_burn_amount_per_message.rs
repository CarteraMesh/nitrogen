use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SetMaxBurnAmountPerMessage {
    pub params: SetMaxBurnAmountPerMessageParams,
}

impl borsh::BorshSerialize for SetMaxBurnAmountPerMessage {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[30, 128, 145, 240, 70, 237, 109, 207])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SetMaxBurnAmountPerMessage {
    pub fn build(
        &self,
        token_controller: Pubkey,
        token_minter: Pubkey,
        program: Pubkey,
        local_token_type: LocalToken,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(5);
        accounts.push(AccountMeta::new_readonly(token_controller, true));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[b"local_token", local_token_type.mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
