use {
    crate::{
        idl::{Idl, IdlPda, IdlPdaSeed},
        legacy_idl::{LegacyIdl, LegacyIdlInstructionDiscriminant},
        util::idl_type_to_rust_type,
    },
    askama::Template,
    heck::{ToSnakeCase, ToUpperCamelCase},
    sha2::{Digest, Sha256},
    std::{collections::HashSet, fmt::Display},
};

#[allow(dead_code)]
#[derive(Debug)]
#[warn(clippy::expect_fun_call)]
pub struct Discriminator(pub Vec<u8>);

impl Display for Discriminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = self
            .0
            .iter()
            .map(|b| format!("{b}"))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "&[{}]", raw)
    }
}

#[derive(Debug)]
pub struct InstructionData {
    pub struct_name: String,
    pub module_name: String,
    pub discriminator: Discriminator,
    pub args: Vec<ArgumentData>,
    pub accounts: Vec<AccountMetaData>,
    pub param_types: HashSet<InstructionParamType>,
    pub requires_imports: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ArgumentData {
    pub name: String,
    pub rust_type: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct InstructionParamType {
    pub name: String,
    pub rust_type: String,
}

impl InstructionParamType {
    pub fn new(rust_type: String) -> Self {
        let name = format!("{}_type", rust_type.to_snake_case());
        Self { name, rust_type }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AccountMetaData {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
    pub address: Option<String>,
    pub pda: Option<Pda>,
}

#[derive(Debug)]
pub struct Pda {
    pub seeds: Vec<PdaSeed>,
}

/// PdaParamType are used to build instructions from a parameter outside the
/// instruction parameters.
/// Example IDL with a PDA path param
/// ```
/// {
///   "name": "token_pair",
///   "writable": true,
///   "pda": {
///     "seeds": [
///       {
///         "kind": "const",
///         "value": [116]
///       },
///       {
///         "kind": "account",
///         "path": "token_pair.remote_domain",
///         "account": "TokenPair"
///       },
///       {
///         "kind": "account",
///         "path": "token_pair.remote_token",
///         "account": "TokenPair"
///       }
///     ]
///   }
/// }
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct PdaParamType {
    pub name: String,
    pub rust_type: Option<String>,
    pub field: String,
}

impl Display for PdaParamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.as_ref()", self.name, self.field)
    }
}

impl PdaParamType {
    pub fn maybe_type(rust_type: &Option<String>, path: &str) -> Option<Self> {
        let name = if let Some(rt) = rust_type {
            format!("{}_type", rt.to_snake_case())
        } else {
            "self.params".to_string()
        };
        let field = if path.contains(".") {
            let v: Vec<&str> = path.split(".").collect();
            if v.len() == 2 {
                v[1].to_string()
            } else {
                return None;
            }
        } else {
            return None;
        };
        Some(PdaParamType {
            name,
            rust_type: rust_type.clone(),
            field,
        })
    }
}

impl From<&IdlPda> for Pda {
    fn from(idl_pda: &IdlPda) -> Self {
        Pda {
            seeds: idl_pda.seeds.iter().map(PdaSeed::from).collect(),
        }
    }
}

#[derive(Debug)]
pub struct PdaSeed {
    pub kind: String,
    pub value: Option<SeedData>,
    pub path: Option<String>,
    pub account: Option<String>,
    pub param_type: Option<PdaParamType>,
}

#[derive(Debug)]
pub struct SeedData {
    value: Vec<u8>,
}

impl Display for SeedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = String::from_utf8_lossy(&self.value).to_string();
        if data.is_ascii() {
            write!(f, "b\"{}\"", data)
        } else {
            let raw = self
                .value
                .iter()
                .map(|b| format!("{b}"))
                .collect::<Vec<String>>()
                .join(",");
            write!(f, "&[{}]", raw)
        }
    }
}

impl From<&IdlPdaSeed> for PdaSeed {
    fn from(idl_pda_seed: &IdlPdaSeed) -> Self {
        let param_type: Option<PdaParamType> = if let Some(path) = &idl_pda_seed.path {
            PdaParamType::maybe_type(&idl_pda_seed.account, path)
        } else {
            None
        };
        PdaSeed {
            kind: idl_pda_seed.kind.clone(),
            value: idl_pda_seed
                .value
                .as_ref()
                .map(|v| SeedData { value: v.clone() }),
            path: idl_pda_seed.path.clone(),
            account: idl_pda_seed.account.clone(),
            param_type,
        }
    }
}

#[derive(Template)]
#[template(path = "instructions_struct.askama", escape = "none", ext = ".askama")]
pub struct InstructionsStructTemplate<'a> {
    pub instruction: &'a InstructionData,
}

#[derive(Template)]
#[template(path = "instructions_mod.askama", escape = "none", ext = ".askama")]
pub struct InstructionsModTemplate<'a> {
    pub instructions: &'a Vec<InstructionData>,
    pub decoder_name: String,
    pub program_instruction_enum: String,
}

pub fn legacy_process_instructions(idl: &LegacyIdl) -> Vec<InstructionData> {
    let mut instructions_data = Vec::new();

    for instruction in &idl.instructions {
        let mut requires_imports = false;
        let module_name = instruction.name.to_snake_case();
        let struct_name = instruction.name.to_upper_camel_case();
        let discriminator = Discriminator(legacy_compute_instruction_discriminator(
            &instruction.name.to_snake_case(),
            instruction.discriminant.as_ref(),
        ));

        let mut args = Vec::new();
        for arg in &instruction.args {
            let rust_type = idl_type_to_rust_type(&arg.type_);
            if rust_type.1 {
                requires_imports = true;
            }
            args.push(ArgumentData {
                name: arg.name.to_snake_case(),
                rust_type: rust_type.0,
            });
        }

        let mut accounts = Vec::new();
        for account in &instruction.accounts {
            accounts.push(AccountMetaData {
                name: account.name.to_snake_case(),
                is_mut: account.is_mut,
                is_signer: account.is_signer,
                is_optional: account.is_optional.unwrap_or(false),
                address: None,
                pda: None,
            });
        }

        instructions_data.push(InstructionData {
            struct_name,
            module_name,
            discriminator,
            args,
            accounts,
            param_types: HashSet::with_capacity(0),
            requires_imports,
        });
    }

    instructions_data
}

pub fn process_instructions(idl: &Idl) -> Vec<InstructionData> {
    let mut instructions_data = Vec::new();

    for instruction in &idl.instructions {
        let mut requires_imports = false;
        let module_name = instruction.name.to_snake_case();
        let struct_name = instruction.name.to_upper_camel_case();
        let discriminator = Discriminator(instruction.discriminator.to_vec());

        let mut args = Vec::new();
        for arg in &instruction.args {
            let rust_type = idl_type_to_rust_type(&arg.type_);
            if rust_type.1 {
                requires_imports = true;
            }
            args.push(ArgumentData {
                name: arg.name.to_snake_case(),
                rust_type: rust_type.0,
            });
        }

        let mut accounts = Vec::with_capacity(instruction.accounts.len());
        let mut param_types: HashSet<InstructionParamType> =
            HashSet::with_capacity(instruction.accounts.len());
        for account in &instruction.accounts {
            accounts.push(AccountMetaData {
                name: account.name.to_snake_case(),
                is_mut: account.writable.unwrap_or(false),
                is_signer: account.signer.unwrap_or(false),
                // TODO: Check
                is_optional: false,
                address: account.address.clone(),
                pda: if let Some(pda) = &account.pda {
                    let p = Pda::from(pda);
                    for seed in &p.seeds {
                        if let Some(ref pt) = seed.param_type {
                            if let Some(ref rt) = pt.rust_type {
                                param_types.insert(InstructionParamType::new(rt.clone()));
                            }
                        }
                    }
                    Some(Pda::from(pda))
                } else {
                    None
                },
            });
        }

        instructions_data.push(InstructionData {
            struct_name,
            module_name,
            discriminator,
            args,
            accounts,
            param_types,
            requires_imports,
        });
    }

    instructions_data
}

fn legacy_compute_instruction_discriminator(
    instruction_name: &str,
    option_discriminant: Option<&LegacyIdlInstructionDiscriminant>,
) -> Vec<u8> {
    if let Some(discriminant) = option_discriminant {
        discriminant.value.to_be_bytes().to_vec()
    } else {
        let mut hasher = Sha256::new();
        let discriminator_input = format!("global:{}", instruction_name);
        hasher.update(discriminator_input.as_bytes());
        let hash = hasher.finalize();
        hash[..8].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pda_param_type() {
        let rt = Some(String::from("TokenPair"));
        let pt = PdaParamType::maybe_type(&rt, "token_pair.remote_domain");
        assert!(pt.is_some());
        let pt = pt.unwrap();
        assert!(pt.rust_type.is_some());
        assert_eq!("token_pair_type.remote_domain.as_ref()", format!("{pt}"));
    }

    #[test]
    fn test_pda_param_types_set() {
        let rt = Some(String::from("TokenPair"));
        let mut set = HashSet::new();
        let pt = PdaParamType::maybe_type(&rt, "token_pair.remote_domain");
        assert!(pt.is_some());
        let pt = pt.unwrap();
        set.insert(pt.clone());
        set.insert(pt.clone());
        assert_eq!(1, set.len());
    }
}
