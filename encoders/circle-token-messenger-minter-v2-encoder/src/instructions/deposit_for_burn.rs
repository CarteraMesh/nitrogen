use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct DepositForBurn {
    pub params: DepositForBurnParams,
}

impl borsh::BorshSerialize for DepositForBurn {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[215, 60, 61, 46, 114, 55, 128, 176])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl DepositForBurn {
    pub fn build(
        &self,
        owner: Pubkey,
        event_rent_payer: Pubkey,
        burn_token_account: Pubkey,
        message_transmitter: Pubkey,
        token_messenger: Pubkey,
        remote_token_messenger: Pubkey,
        token_minter: Pubkey,
        burn_token_mint: Pubkey,
        message_sent_event_data: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(18);
        accounts.push(AccountMeta::new_readonly(owner, true));
        accounts.push(AccountMeta::new(event_rent_payer, true));
        accounts.push(crate::derive_pda(&[b"sender_authority"], &crate::ID, true));
        accounts.push(AccountMeta::new(burn_token_account, false));
        accounts.push(crate::derive_pda(
            &[b"denylist_account", owner.as_ref()],
            &crate::ID,
            true,
        ));
        accounts.push(AccountMeta::new(message_transmitter, false));
        accounts.push(AccountMeta::new_readonly(token_messenger, false));
        accounts.push(AccountMeta::new_readonly(remote_token_messenger, false));
        accounts.push(AccountMeta::new_readonly(token_minter, false));
        accounts.push(crate::derive_pda(
            &[b"local_token", burn_token_mint.as_ref()],
            &crate::ID,
            false,
        ));
        accounts.push(AccountMeta::new(burn_token_mint, false));
        accounts.push(AccountMeta::new(message_sent_event_data, true));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("CCTPV2Sm4AdWt5296sk4P66VBZ7bEhcARwFaaS9YPbeC"),
            false,
        ));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("CCTPV2vPZJS2u2BBsUoscuikbYjnpFmbFsvVuJdgUMQe"),
            false,
        ));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
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
