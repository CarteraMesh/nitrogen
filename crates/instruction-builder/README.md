# nitrogen instruction builder

[![Crates.io](https://img.shields.io/crates/v/nitrogen-instruction-builder.svg)](https://crates.io/crates/nitrogen-instruction-builder)
[![CI](https://github.com/CarteraMesh/nitrogen/workflows/test/badge.svg)](https://github.com/CarteraMesh/nitrogen/actions)

Convenience utilities around Solana [Instructions](https://docs.rs/solana-instruction/latest/solana_instruction/struct.Instruction.html)

## Features

- `InstructionBuilder` - Build Solana instructions with Borsh-serialized data

## Usage

```rust,no_run
use nitrogen_instruction_builder::InstructionBuilder;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
use borsh::BorshSerialize;

#[derive(BorshSerialize)]
struct MyParams {
    amount: u64,
}

const PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("So11111111111111111111111111111111111111112");
// Build a single instruction
let ix = InstructionBuilder::builder()
    .program_id(PROGRAM_ID)
    .accounts(vec![AccountMeta::new(account, true)])
    .params(MyParams { amount: 100 })
    .build()
    .instruction();

```
