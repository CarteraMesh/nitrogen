use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct SendMessage {
    pub params: SendMessageParams,
}

impl borsh::BorshSerialize for SendMessage {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[57, 40, 34, 178, 189, 10, 65, 26])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl SendMessage {
    pub fn build(
        &self,
        event_rent_payer: Pubkey,
        message_transmitter: Pubkey,
        message_sent_event_data: Pubkey,
        sender_program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(6);
        accounts.push(AccountMeta::new(event_rent_payer, true));
        accounts.push(crate::derive_pda(&[b"sender_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(AccountMeta::new(message_sent_event_data, true));
        accounts.push(AccountMeta::new_readonly(sender_program, false));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("11111111111111111111111111111111"),
            false,
        ));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
