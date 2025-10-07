use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct ReclaimEventAccount {
    pub params: ReclaimEventAccountParams,
}

impl borsh::BorshSerialize for ReclaimEventAccount {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[94, 198, 180, 159, 131, 236, 15, 174])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl ReclaimEventAccount {
    pub fn build(
        &self,
        payee: Pubkey,
        message_transmitter: Pubkey,
        message_sent_event_data: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(3);
        accounts.push(AccountMeta::new(payee, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(AccountMeta::new(message_sent_event_data, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
