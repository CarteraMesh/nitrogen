use {
    super::super::types::*,
    nitrogen_instruction_builder::{InstructionBuilder, derive_pda},
    solana_instruction::AccountMeta,
    solana_pubkey::Pubkey,
};

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
    pub fn accounts(
        self,
        payee: Pubkey,
        message_sent_event_data: Pubkey,
    ) -> InstructionBuilder<Self> {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(3);
        accounts.push(AccountMeta::new(payee, true));
        accounts.push(derive_pda(&[b"message_transmitter"], &crate::ID, false));
        accounts.push(AccountMeta::new(message_sent_event_data, false));
        InstructionBuilder::builder()
            .accounts(accounts)
            .program_id(crate::ID)
            .params(self)
            .build()
    }
}
