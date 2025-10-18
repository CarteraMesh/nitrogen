#[cfg(not(feature = "blocking"))]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
#[cfg(feature = "blocking")]
use solana_rpc_client::rpc_client::RpcClient;
use {
    crate::{Error, Result},
    solana_account::Account,
    solana_account_decoder::parse_address_lookup_table::{
        LookupTableAccountType,
        parse_address_lookup_table,
    },
    solana_message::AddressLookupTableAccount,
    solana_pubkey::Pubkey,
    std::str::FromStr,
    tracing::debug,
};

#[cfg(feature = "blocking")]
fn get_multiple_accts(lookup_tables: &[Pubkey], rpc: &RpcClient) -> Result<Vec<Option<Account>>> {
    rpc.get_multiple_accounts(lookup_tables)
        .map_err(|e| Error::SolanaRpcError(format!("failed to get lookup table accounts: {e}")))
}

#[cfg(not(feature = "blocking"))]
async fn get_multiple_accts(
    lookup_tables: &[Pubkey],
    rpc: &RpcClient,
) -> Result<Vec<Option<Account>>> {
    rpc.get_multiple_accounts(lookup_tables)
        .await
        .map_err(|e| Error::SolanaRpcError(format!("failed to get lookup table accounts: {e}")))
}

fn process_lookup_tables(
    lookup_tables: &[Pubkey],
    accounts: Vec<Option<Account>>,
) -> Result<Vec<AddressLookupTableAccount>> {
    let mut lookup_tables_state = Vec::with_capacity(accounts.len());

    for (i, maybe_account) in accounts.iter().enumerate() {
        match maybe_account {
            None => tracing::warn!("lookup table account {} not found", lookup_tables[i]),
            Some(account) => {
                // Intentionally left here for future debugging if needed
                // let data =
                //     solana_address_lookup_table_interface::state::AddressLookupTable::deserialize(
                //         account.data(),
                //     )
                //     .unwrap();
                // let encoded: String =
                //     BASE64_STANDARD.encode(data.serialize_for_tests().unwrap().as_slice());
                // eprintln!("{} - {}", lookup_tables[i], encoded);

                let table_type = parse_address_lookup_table(account.data.as_ref())?;
                match table_type {
                    LookupTableAccountType::Uninitialized => {
                        tracing::warn!("lookup table {} is uninitialized", lookup_tables[i])
                    }
                    LookupTableAccountType::LookupTable(table) => {
                        if table.addresses.is_empty() {
                            tracing::warn!(
                                "lookup table addresses are empty for account {}",
                                lookup_tables[i]
                            );
                            continue;
                        }
                        let mut addresses = Vec::with_capacity(table.addresses.len());
                        for a in table.addresses.iter() {
                            addresses.push(Pubkey::from_str(a)?);
                        }
                        lookup_tables_state.push(AddressLookupTableAccount {
                            key: lookup_tables[i],
                            addresses,
                        });
                    }
                }
            }
        }
    }
    debug!(
        "found {} valid lookup table state accounts",
        lookup_tables_state.len()
    );
    Ok(lookup_tables_state)
}

/// Fetches lookup tables from the Solana blockchain.
#[cfg(not(feature = "blocking"))]
pub async fn fetch_lookup_tables(
    lookup_tables: &[Pubkey],
    rpc: &RpcClient,
) -> Result<Vec<AddressLookupTableAccount>> {
    if lookup_tables.is_empty() {
        return Ok(Vec::with_capacity(0));
    }
    debug!(
        "fetching {} accounts for lookup table state",
        lookup_tables.len()
    );
    let accounts = get_multiple_accts(lookup_tables, rpc).await?;
    process_lookup_tables(lookup_tables, accounts)
}

/// Fetches lookup tables from the Solana blockchain.
#[cfg(feature = "blocking")]
pub fn fetch_lookup_tables(
    lookup_tables: &[Pubkey],
    rpc: &RpcClient,
) -> Result<Vec<AddressLookupTableAccount>> {
    if lookup_tables.is_empty() {
        return Ok(Vec::with_capacity(0));
    }
    let accounts = get_multiple_accts(lookup_tables, rpc)?;
    process_lookup_tables(lookup_tables, accounts)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        base64::prelude::*,
        solana_account::{AccountSharedData, WritableAccount},
        solana_address_lookup_table_interface::{
            program::ID as LOOKUP_TABLE_PROGRAM_ID,
            state::AddressLookupTable,
        },
    };
    const NOT_INITIALIZED: Pubkey =
        solana_pubkey::pubkey!("3W6YcoQyFcrSo6K9vixhM2Cfvtjv4KeKSH1FaEKJF1Ug");
    const NOT_INITIALIZED_DATA: &str =
        "AQAAAP//////////AAAAAAAAAAAAAQ7hKwIsEP/WoRK4pP1+ELuGsEsjs8K1lkPsgzEtmy39AAA=";

    const INITIALIZED: Pubkey =
        solana_pubkey::pubkey!("FNK9gB5E3cntDRiy3LHwtwQC6qhbVgdBLBMqjRZLEYiK");
    const INITIALIZED_DATA: &str = r#"AQAAAP//////////xWHAGAAAAAAAAQ7hKwIsEP/WoRK4pP1+ELuGsEsjs8K1lkPsgzEtmy39AAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAATtELLORIVfxOpM9ATQoLQMrX/7NAaLb8bd5BgjfAC6nc0qlNf/0kfkCG8En13LFGRXgKeyZs/EkFm0noup/XR8="#;
    const EXPECTED_TABLE: [Pubkey; 3] = [
        solana_pubkey::pubkey!("So11111111111111111111111111111111111111112"),
        solana_pubkey::pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),
        solana_pubkey::pubkey!("8m3uKEn4fMPNVr7nv6RmQYktT4zRqEZzhuZDpG8hQZT4"),
    ];

    fn convert(data: &str) -> anyhow::Result<Account> {
        let data: Vec<u8> = BASE64_STANDARD.decode(data)?;
        let address_lookup = AddressLookupTable::deserialize(&data)?;
        let account_shared_data = AccountSharedData::create(
            1,
            address_lookup.serialize_for_tests()?,
            LOOKUP_TABLE_PROGRAM_ID,
            false,
            0,
        );
        Ok(account_shared_data.into())
    }
    #[test]
    fn test_empty_table() -> anyhow::Result<()> {
        let result = process_lookup_tables(&[], vec![])?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn test_not_initialized() -> anyhow::Result<()> {
        let account: Account = convert(NOT_INITIALIZED_DATA)?;
        let result = process_lookup_tables(&[NOT_INITIALIZED], vec![Some(account)])?;
        assert!(result.is_empty());

        Ok(())
    }

    #[test]
    fn test_initialized() -> anyhow::Result<()> {
        let account: Account = convert(INITIALIZED_DATA)?;
        let result = process_lookup_tables(&[INITIALIZED], vec![Some(account)])?;
        assert_eq!(1, result.len());
        assert_eq!(result[0].key, INITIALIZED);
        assert_eq!(result[0].addresses, EXPECTED_TABLE);
        Ok(())
    }

    #[test]
    fn test_mixed() -> anyhow::Result<()> {
        let accounts: Vec<Option<Account>> = vec![
            Some(convert(INITIALIZED_DATA)?),
            Some(convert(NOT_INITIALIZED_DATA)?),
        ];
        let result = process_lookup_tables(&[INITIALIZED, NOT_INITIALIZED], accounts)?;
        assert_eq!(1, result.len());
        assert_eq!(result[0].key, INITIALIZED);
        assert_eq!(result[0].addresses, EXPECTED_TABLE);
        Ok(())
    }
}
