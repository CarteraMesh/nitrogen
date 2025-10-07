use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct HandleReceiveFinalizedMessage {
    pub params: HandleReceiveMessageParams,
}

impl borsh::BorshSerialize for HandleReceiveFinalizedMessage {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[186, 252, 239, 70, 86, 180, 110, 95])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl HandleReceiveFinalizedMessage {
    pub fn build(
        &self,
        token_messenger: Pubkey,
        remote_token_messenger: Pubkey,
        token_minter: Pubkey,
        token_pair: Pubkey,
        recipient_token_account: Pubkey,
        program: Pubkey,
        local_token_type: LocalToken,
        token_messenger_type: TokenMessenger,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(12);
        accounts.push(crate::derive_pda(
            &[b"message_transmitter_authority", &[
                166, 95, 200, 29, 15, 239, 168, 134, 12, 179, 184, 63, 8, 155, 2, 36, 190, 138,
                102, 135, 183, 174, 73, 245, 148, 192, 185, 180, 215, 233, 56, 147,
            ]],
            &crate::ID,
            true,
        ));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(AccountMeta::new_readonly(remote_token_messenger, false));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[b"local_token", local_token_type.mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new_readonly(token_pair, false));
        accounts.push(crate::derive_pda(
            &[
                token_messenger_type.fee_recipient.as_ref(),
                &[
                    6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172,
                    28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
                ],
                local_token_type.mint.as_ref(),
            ],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new(recipient_token_account, false));
        accounts.push(crate::derive_pda(
            &[b"custody", local_token_type.mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            false,
        ));
        accounts.push(crate::derive_pda(&[b"__event_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new_readonly(program, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
