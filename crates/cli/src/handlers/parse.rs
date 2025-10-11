use {
    crate::{
        accounts::{AccountsModTemplate, AccountsStructTemplate, process_accounts},
        instructions::{InstructionsModTemplate, InstructionsStructTemplate, process_instructions},
        types::{TypeStructTemplate, process_types},
        util::{is_big_array, read_idl},
    },
    anyhow::{Result, bail},
    askama::Template,
    heck::{ToKebabCase, ToSnakeCase, ToUpperCamelCase},
    std::fs::{self},
};

pub fn parse(path: String, output: String, crate_name: Option<String>) -> Result<()> {
    let (accounts_data, instructions_data, types_data, program_name, program_id) =
        match read_idl(&path) {
            Ok(idl) => {
                let accounts_data = process_accounts(&idl);
                let instructions_data = process_instructions(&idl);
                let types_data = process_types(&idl);
                let program_name = idl.metadata.name;
                let program_id = idl.address;

                (
                    accounts_data,
                    instructions_data,
                    types_data,
                    program_name,
                    program_id,
                )
            }
            Err(idl_err) => {
                bail!("{idl_err}");
            }
        };

    let encoder_name = format!("{}Encoder", program_name.to_upper_camel_case());
    let program_struct_name = format!("{}Account", program_name.to_upper_camel_case());
    let program_instruction_enum = format!("{}Instruction", program_name.to_upper_camel_case());

    let crate_dir = match &crate_name {
        Some(name) => format!("{}/{}", output.trim_end_matches('/'), name),
        None => format!(
            "{}/{}_encoder",
            output.trim_end_matches('/'),
            program_name.to_snake_case()
        ),
    };

    fs::create_dir_all(&crate_dir).expect("Failed to create encoder directory");

    let src_dir = if crate_name.is_some() {
        format!("{}/src", crate_dir)
    } else {
        crate_dir.clone()
    };

    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    let needs_big_array = types_data.iter().any(|type_data| {
        type_data.fields.iter().any(|field| {
            field.rust_type.starts_with("[")
                && field.rust_type.ends_with("]")
                && is_big_array(&field.rust_type)
        })
    });

    // Generate types
    let types_dir = format!("{}/types", src_dir);
    fs::create_dir_all(&types_dir).expect("Failed to create types directory");

    for type_data in &types_data {
        let template = TypeStructTemplate { type_data };
        let rendered = template
            .render()
            .expect("Failed to render type struct template");
        let filename = format!("{}/{}.rs", types_dir, type_data.name.to_snake_case());
        fs::write(&filename, rendered).expect("Failed to write type struct file");
        println!("Generated {}", filename);
    }

    let types_mod_content = types_data
        .iter()
        .map(|type_data| {
            format!(
                "pub mod {};\npub use {}::*;",
                type_data.name.to_snake_case(),
                type_data.name.to_snake_case()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let types_mod_filename = format!("{}/mod.rs", types_dir);
    fs::write(&types_mod_filename, types_mod_content).expect("Failed to write types mod file");
    println!("Generated {}", types_mod_filename);

    // Generate Accounts

    let accounts_dir = format!("{}/accounts", src_dir);
    fs::create_dir_all(&accounts_dir).expect("Failed to create accounts directory");

    for account in &accounts_data {
        let template = AccountsStructTemplate { account };
        let rendered = template
            .render()
            .expect("Failed to render account struct template");
        let filename = format!("{}/{}.rs", accounts_dir, account.module_name);
        fs::write(&filename, rendered).expect("Failed to write account struct file");
        println!("Generated {}", filename);
    }

    let accounts_mod_template = AccountsModTemplate {
        accounts: &accounts_data,
        decoder_name: encoder_name.clone(),
        program_struct_name: program_struct_name.clone(),
    };
    let accounts_mod_rendered = accounts_mod_template
        .render()
        .expect("Failed to render mod file");
    let accounts_mod_filename = format!("{}/mod.rs", accounts_dir);

    fs::write(&accounts_mod_filename, accounts_mod_rendered)
        .expect("Failed to write accounts mod file");
    println!("Generated {}", accounts_mod_filename);

    // Generate Instructions

    let instructions_dir = format!("{}/instructions", src_dir);
    fs::create_dir_all(&instructions_dir).expect("Failed to create instructions directory");

    for instruction in &instructions_data {
        let template = InstructionsStructTemplate { instruction };
        let rendered = template
            .render()
            .expect("Failed to render instruction struct template");
        let filename = format!("{}/{}.rs", instructions_dir, instruction.module_name);
        fs::write(&filename, rendered).expect("Failed to write instruction struct file");
        println!("Generated {}", filename);
    }

    let instructions_mod_template = InstructionsModTemplate {
        instructions: &instructions_data,
        decoder_name: encoder_name.clone(),
        program_instruction_enum: program_instruction_enum.clone(),
    };
    let instructions_mod_rendered = instructions_mod_template
        .render()
        .expect("Failed to render instruction mod file");
    let instructions_mod_filename = format!("{}/mod.rs", instructions_dir);

    fs::write(&instructions_mod_filename, instructions_mod_rendered)
        .expect("Failed to write instructions mod file");

    println!("Generated {}", instructions_mod_filename);
    let crate_package_name = match &crate_name {
        Some(c) => c.clone(),
        None => format!("{program_name}-encoder").to_kebab_case(),
    };

    if crate_name.is_some() {
        let mut lib_rs_content = format!(
            r#"use {{solana_instruction::AccountMeta, solana_pubkey::{{declare_id, Pubkey}}}};
pub struct {encoder_name};
pub mod accounts;
pub mod instructions;
pub mod types;

declare_id!("{program_id}");

"#,
            encoder_name = encoder_name,
            program_id = program_id
        );
        lib_rs_content.push_str(
            r#"
pub(crate) fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey, read_only: bool) -> AccountMeta {
  if read_only {
   AccountMeta::new_readonly(Pubkey::find_program_address(seeds, program_id).0, false)
  } else {
  AccountMeta::new(Pubkey::find_program_address(seeds, program_id).0, false)
 }
}            "#,
        );
        let lib_rs_filename = format!("{}/lib.rs", src_dir);
        fs::write(&lib_rs_filename, lib_rs_content).expect("Failed to write lib.rs file");
        println!("Generated {}", lib_rs_filename);

        let cargo_toml_content = format!(
            r#"[package]
name = "{crate_package_name}"
version = "0.1.0"
edition = {{ workspace = true }}
description = "{crate_package_name}"
license = {{ workspace = true }
readme = "README.md"
repository = {{ workspace = true }}
keywords = ["solana", "idl"]
categories = ["encoding"]

[dependencies]
bon = {{ workspace = true }}
borsh = {{ workspace = true }}
serde = {{ workspace = true }}
solana-instruction = {{ workspace = true }}
solana-pubkey = {{ workspace = true }}
{big_array}

[lints]
workspace = true
"#,
            crate_package_name = crate_package_name,
            big_array = if needs_big_array {
                "serde-big-array = { workspace = true }"
            } else {
                ""
            }
        );
        let cargo_toml_filename = format!("{}/Cargo.toml", crate_dir);
        fs::write(&cargo_toml_filename, cargo_toml_content)
            .expect("Failed to write Cargo.toml file");
        println!("Generated {}", cargo_toml_filename);
    } else {
        let mod_rs_content = format!(
            "pub struct {encoder_name};\npub mod accounts;\npub mod instructions;\npub mod types;",
            encoder_name = encoder_name
        );
        let mod_rs_filename = format!("{}/mod.rs", src_dir);
        fs::write(&mod_rs_filename, mod_rs_content).expect("Failed to write mod.rs file");
        println!("Generated {}", mod_rs_filename);
    }

    Ok(())
}
