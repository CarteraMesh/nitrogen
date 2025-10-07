use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct UnlinkTokenPair {
    pub params: UninkTokenPairParams,
}

impl borsh::BorshSerialize for UnlinkTokenPair {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[52, 198, 100, 114, 104, 174, 85, 58])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl UnlinkTokenPair {
    pub fn build(
        &self,
        payee: Pubkey,
        token_controller: Pubkey,
        token_minter: Pubkey,
        program: Pubkey,
        token_pair_type: TokenPair,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(6);
        accounts.push(AccountMeta::new(payee, true));
        accounts.push(AccountMeta::new_readonly(token_controller, true));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[
                b"token_pair",
                token_pair_type.remote_domain.to_le_bytes().as_ref(),
                token_pair_type.remote_token.as_ref(),
            ],
            &crate::ID,
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
