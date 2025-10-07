use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SetMinFee {
    pub params: SetMinFeeParams,
}

impl borsh::BorshSerialize for SetMinFee {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[114, 198, 35, 3, 41, 196, 194, 246])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SetMinFee {
    pub fn build(
        &self,
        min_fee_controller: Pubkey,
        token_messenger: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(4);
        accounts.push(AccountMeta::new_readonly(min_fee_controller, true));
        accounts.push(AccountMeta::new(token_messenger, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
