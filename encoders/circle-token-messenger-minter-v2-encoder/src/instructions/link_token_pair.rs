use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct LinkTokenPair {
    pub params: LinkTokenPairParams,
}

impl borsh::BorshSerialize for LinkTokenPair {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[68, 162, 24, 104, 125, 46, 130, 12])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl LinkTokenPair {
    pub fn build(
        &self,
        payer: Pubkey,
        token_controller: Pubkey,
        token_minter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(7);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(token_controller, true));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[
                b"token_pair",
                self.params.remote_domain.to_le_bytes().as_ref(),
                self.params.remote_token.as_ref(),
            ],
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
