use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct RemoveRemoteTokenMessenger {
    pub params: RemoveRemoteTokenMessengerParams,
}

impl borsh::BorshSerialize for RemoveRemoteTokenMessenger {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[65, 114, 66, 85, 169, 98, 177, 146])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl RemoveRemoteTokenMessenger {
    pub fn build(
        &self,
        payee: Pubkey,
        owner: Pubkey,
        token_messenger: Pubkey,
        program: Pubkey,
        remote_token_messenger_type: RemoteTokenMessenger,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(6);
        accounts.push(AccountMeta::new(payee, true));
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(crate::derive_pda(
            &[
                b"remote_token_messenger",
                remote_token_messenger_type.domain.to_le_bytes().as_ref(),
            ],
            &crate::ID,
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
