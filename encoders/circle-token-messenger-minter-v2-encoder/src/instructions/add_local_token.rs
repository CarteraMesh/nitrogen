use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct AddLocalToken {
    pub params: AddLocalTokenParams,
}

impl borsh::BorshSerialize for AddLocalToken {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[213, 199, 205, 18, 98, 124, 73, 198])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl AddLocalToken {
    pub fn build(
        &self,
        payer: Pubkey,
        token_controller: Pubkey,
        token_minter: Pubkey,
        local_token_mint: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(10);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(token_controller, true));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[b"local_token", local_token_mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(crate::derive_pda(
            &[b"custody", local_token_mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new_readonly(local_token_mint, false));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
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
