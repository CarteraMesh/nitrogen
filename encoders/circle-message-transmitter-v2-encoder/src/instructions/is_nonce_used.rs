use {solana_instruction::AccountMeta, solana_pubkey::Pubkey};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct IsNonceUsed {}

impl borsh::BorshSerialize for IsNonceUsed {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[144, 72, 107, 148, 35, 218, 31, 187])?;
        Ok(())
    }
}

impl IsNonceUsed {
    pub fn build(&self, used_nonce: Pubkey) -> solana_instruction::Instruction {
        let mut accounts: Vec<AccountMeta> = Vec::with_capacity(1);
        accounts.push(AccountMeta::new_readonly(used_nonce, false));
        solana_instruction::Instruction::new_with_borsh(crate::ID, self, accounts)
    }
}
