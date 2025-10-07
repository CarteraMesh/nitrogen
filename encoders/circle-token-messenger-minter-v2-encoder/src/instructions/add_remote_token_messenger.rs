use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct AddRemoteTokenMessenger {
    pub params: AddRemoteTokenMessengerParams,
}

impl borsh::BorshSerialize for AddRemoteTokenMessenger {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[12, 149, 172, 165, 111, 202, 24, 33])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl AddRemoteTokenMessenger {
    pub fn build(
        &self,
        payer: Pubkey,
        owner: Pubkey,
        token_messenger: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(7);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(crate::derive_pda(
            &[
                b"remote_token_messenger",
                self.params.domain.to_le_bytes().as_ref(),
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
