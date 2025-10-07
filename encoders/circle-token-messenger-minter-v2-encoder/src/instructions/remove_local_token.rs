use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct RemoveLocalToken {
    pub params: RemoveLocalTokenParams,
}

impl borsh::BorshSerialize for RemoveLocalToken {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[27, 43, 66, 170, 188, 44, 109, 97])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl RemoveLocalToken {
    pub fn build(
        &self,
        payee: Pubkey,
        token_controller: Pubkey,
        token_minter: Pubkey,
        custody_token_mint: Pubkey,
        program: Pubkey,
        local_token_type: LocalToken,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(9);
        accounts.push(AccountMeta::new(payee, true));
        accounts.push(AccountMeta::new_readonly(token_controller, true));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[b"local_token", local_token_type.mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(crate::derive_pda(
            &[b"custody", local_token_type.mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new(custody_token_mint, false));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
