use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SetSignatureThreshold {
    pub params: SetSignatureThresholdParams,
}

impl borsh::BorshSerialize for SetSignatureThreshold {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[163, 19, 154, 168, 82, 209, 214, 219])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SetSignatureThreshold {
    pub fn build(
        &self,
        attester_manager: Pubkey,
        message_transmitter: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(4);
        accounts.push(AccountMeta::new_readonly(attester_manager, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
