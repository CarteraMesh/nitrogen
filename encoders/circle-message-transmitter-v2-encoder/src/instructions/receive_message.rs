use {
    super::super::types::*,
    nitrogen_instruction_builder::{InstructionBuilder, derive_pda},
    solana_instruction::AccountMeta,
    solana_pubkey::Pubkey,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct ReceiveMessage {
    pub params: ReceiveMessageParams,
}

impl borsh::BorshSerialize for ReceiveMessage {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[38, 144, 127, 225, 31, 225, 238, 25])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl ReceiveMessage {
    pub fn accounts(
        self,
        payer: Pubkey,
        caller: Pubkey,
        message_transmitter: Pubkey,
        used_nonce: Pubkey,
        receiver: Pubkey,
        program: Pubkey,
    ) -> InstructionBuilder<Self> {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(9);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(caller, true));
        accounts.push(derive_pda(
            &[b"message_transmitter_authority", receiver.as_ref()],
            &crate::ID,
            true,
        ));
        accounts.push(AccountMeta::new_readonly(message_transmitter, false));
        accounts.push(AccountMeta::new(used_nonce, false));
        accounts.push(AccountMeta::new_readonly(receiver, false));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("11111111111111111111111111111111"),
            false,
        ));
        accounts.push(derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        InstructionBuilder::builder()
            .accounts(accounts)
            .program_id(crate::ID)
            .params(self)
            .build()
    }
}
