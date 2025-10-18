# nitrogen instruction builder

[![Crates.io](https://img.shields.io/crates/v/nitrogen-instruction-builder.svg)](https://crates.io/crates/nitrogen-instruction-builder)
[![CI](https://github.com/CarteraMesh/nitrogen/workflows/test/badge.svg)](https://github.com/CarteraMesh/nitrogen/actions)

Wrapper around Solana SDK types for building instructions and transactions.

## Features

- `InstructionBuilder` - Build Solana instructions with Borsh-serialized data
- `TransactionBuilder` - Build and send transactions with simulation support
- `blocking` feature - Use blocking RPC client instead of async

## Usage

```rust,no_run
use nitrogen_instruction_builder::{InstructionBuilder, TransactionBuilder};
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

// Build and send a transaction
let sig = InstructionBuilder::builder()
    .program_id(PROGRAM_ID)
    .accounts(vec![AccountMeta::new(account, true)])
    .params(MyParams { amount: 100 })
    .build()
    .tx()
    .send(&rpc, Some(&payer), &[&signer])
    .await?;
```
