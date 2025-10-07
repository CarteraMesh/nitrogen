use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct DisableAttester {
    pub params: DisableAttesterParams,
}

impl borsh::BorshSerialize for DisableAttester {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[61, 171, 131, 95, 172, 15, 227, 229])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl DisableAttester {
    pub fn build(
        &self,
        payer: Pubkey,
        attester_manager: Pubkey,
        message_transmitter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(6);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(attester_manager, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("11111111111111111111111111111111"),
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
