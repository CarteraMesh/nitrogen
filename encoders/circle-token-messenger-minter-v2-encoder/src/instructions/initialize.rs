use {super::super::types::*, solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Initialize {
    pub params: InitializeParams,
}

impl borsh::BorshSerialize for Initialize {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[175, 175, 109, 31, 13, 152, 155, 237])?;
        self.params.serialize(writer)?;
        Ok(())
    }
}

impl Initialize {
    pub fn build(
        &self,
        payer: Pubkey,
        upgrade_authority: Pubkey,
        token_messenger_minter_program_data: Pubkey,
        program: Pubkey,
    ) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(10);
        accounts.push(AccountMeta::new(payer, true));
        accounts.push(AccountMeta::new_readonly(upgrade_authority, true));
        accounts.push(crate::derive_pda(&[b"sender_authority"], &crate::ID, true));
        accounts.push(crate::derive_pda(&[b"token_messenger"], &crate::ID, false));
        accounts.push(crate::derive_pda(&[b"token_minter"], &crate::ID, false));
        accounts.push(AccountMeta::new_readonly(
            token_messenger_minter_program_data,
            false,
        ));
        accounts.push(AccountMeta::new_readonly(
            solana_pubkey::pubkey!("CCTPV2vPZJS2u2BBsUoscuikbYjnpFmbFsvVuJdgUMQe"),
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
