use super::{hex_to_vec, addr_to_bytes20};
use crate::prelude::Address;
use ethabi::{Constructor, Contract};
use ethereum_tx_sign::LegacyTransaction;
use serde::Deserialize;
use serde_json::{self, Error as JsonError};
use std::error::Error;
use std::fmt;
use std::fs::{self};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A struct representing an Ethereum smart contract constructor
pub struct ContractConstructor {
    /// The contract's ABI
    pub abi: Contract,
    /// The contract's bytecode
    pub code: Vec<u8>,
}

/// A struct representing a deployed Ethereum smart contract
pub struct DeployedContract {
    /// The contract's ABI
    pub abi: ethabi::Contract,
    /// The contract's address
    pub address: Address,
}

impl DeployedContract {
    pub fn addr_to_bytes20(&self) -> [u8; 20] {
        addr_to_bytes20(&self.address)
    }
}

/// A struct representing an Ethereum smart contract artifact
#[derive(Deserialize)]
struct ExtendedJsonSolidityArtifact {
    /// The contract's ABI
    abi: ethabi::Contract,
    /// The contract's bytecode
    bytecode: String,
}

// ContractConstructor Errors
#[derive(Debug)]
pub enum ContractConstructorError {
    /// An I/O error occurred
    IoError(IoError),
    /// A JSON parsing error occurred
    JsonError(JsonError),
    /// A hexadecimal decoding error occurred
    HexError(hex::FromHexError),
}

impl Error for ContractConstructorError {}

impl fmt::Display for ContractConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractConstructorError::IoError(e) => write!(f, "I/O error: {}", e),
            ContractConstructorError::JsonError(e) => write!(f, "JSON error: {}", e),
            ContractConstructorError::HexError(e) => write!(f, "Hex decoding error: {}", e),
        }
    }
}

impl From<IoError> for ContractConstructorError {
    fn from(error: IoError) -> Self {
        ContractConstructorError::IoError(error)
    }
}

impl From<JsonError> for ContractConstructorError {
    fn from(error: JsonError) -> Self {
        ContractConstructorError::JsonError(error)
    }
}

impl From<hex::FromHexError> for ContractConstructorError {
    fn from(error: hex::FromHexError) -> Self {
        ContractConstructorError::HexError(error)
    }
}

impl ContractConstructor {
    /// Creates a new instance of `ContractConstructor` by reading the contract artifact from the given file path
    ///
    /// # Arguments
    ///
    /// * `artifact_path` - The file path of the contract artifact JSON file
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ContractConstructor` instance if successful, or an `ContractConstructorError` if an error occurred
    pub fn new(artifact_path: &str) -> Result<Self, ContractConstructorError> {
        let json_str = fs::read_to_string(PathBuf::from(artifact_path))?;
        let artifact: ExtendedJsonSolidityArtifact = serde_json::from_str(&json_str)?;
        let code = hex_to_vec(&artifact.bytecode)?;
        Ok(Self {
            abi: artifact.abi,
            code,
        })
    }

    /// Generates a transaction to deploy the contract to the blockchain
    ///
    /// # Arguments
    ///
    /// * `nonce` - A unique nonce value to ensure transaction uniqueness
    /// * `args` - The arguments to pass to the contract constructor, if any
    ///
    /// # Returns
    ///
    /// A `LegacyTransaction` containing the transaction details for submitting to NEAR Workspace
    pub fn deploy_transaction(&self, nonce: u128, args: &[ethabi::Token]) -> LegacyTransaction {
        let data = self
            .abi
            .constructor()
            .unwrap_or(&Constructor { inputs: vec![] })
            .encode_input(self.code.clone(), args)
            .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            to: None,
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }

    /// Same as `compile_from_source` but always recompiles instead of reusing artifacts when they exist.
    ///
    /// # Arguments
    ///
    /// * `sources_root` - A reference to the root directory containing the Solidity source files.
    /// * `artifacts_base_path` - A reference to the base path where the compiled contract artifacts will be stored.
    /// * `contract_file` - A reference to the Solidity source file containing the contract definition.
    /// * `contract_name` - The name of the contract to compile.
    ///
    /// # Returns
    ///
    /// A `CompiledContract` instance.
    ///
    /// # Panics
    ///
    /// This method panics if the Solidity compiler is not installed or if compilation fails.
    pub fn force_compile<P1, P2, P3>(
        sources_root: P1,
        artifacts_base_path: P2,
        contract_file: P3,
        contract_name: &str,
    ) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        compile(&sources_root, &contract_file, &artifacts_base_path);
        Self::compile_from_source(
            sources_root,
            artifacts_base_path,
            contract_file,
            contract_name,
        )
    }

    /// Compiles a Solidity contract and returns a `CompiledContract` instance.
    ///
    /// # Arguments
    ///
    /// * `sources_root` - A reference to the root directory containing the Solidity source files.
    /// * `artifacts_base_path` - A reference to the base path where the compiled contract artifacts will be stored.
    /// * `contract_file` - A reference to the Solidity source file containing the contract definition. Note that this must be relative to `sources_root`.
    /// * `contract_name` - The name of the contract to compile.
    ///
    /// # Returns
    ///
    /// A `CompiledContract` instance.
    ///
    /// # Panics
    ///
    /// This method panics if the Solidity compiler is not installed or if compilation fails.
    pub fn compile_from_source<P1, P2, P3>(
        sources_root: P1,
        artifacts_base_path: P2,
        contract_file: P3,
        contract_name: &str,
    ) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        let bin_file = format!("{}.bin", contract_name);
        let abi_file = format!("{}.abi", contract_name);
        let hex_path = artifacts_base_path.as_ref().join(&bin_file);
        let hex_rep = match std::fs::read_to_string(&hex_path) {
            Ok(hex) => hex,
            Err(_) => {
                // An error occurred opening the file, maybe the contract hasn't been compiled?
                compile(sources_root, contract_file, &artifacts_base_path);
                // If another error occurs, then we can't handle it so we just unwrap.
                std::fs::read_to_string(hex_path).unwrap()
            }
        };
        let code = hex::decode(&hex_rep).unwrap();
        let abi_path = artifacts_base_path.as_ref().join(&abi_file);
        let reader = std::fs::File::open(abi_path).unwrap();
        let abi = ethabi::Contract::load(reader).unwrap();

        Self { abi, code }
    }

    /// Parses an extended JSON solidity artifact from the given contract path and returns a ContractConstructor object containing the artifact's ABI and bytecode.
    ///
    /// # Arguments
    ///
    /// * `contract_path` - Path to the contract file containing the extended JSON solidity artifact
    ///
    /// # Returns
    ///
    /// A ContractConstructor object containing the contract's ABI and bytecode
    pub fn compile_from_extended_json<P>(contract_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let reader = std::fs::File::open(contract_path).unwrap();
        let contract: ExtendedJsonSolidityArtifact = serde_json::from_reader(reader).unwrap();

        Self {
            abi: contract.abi,
            code: hex::decode(&contract.bytecode[2..]).unwrap(),
        }
    }

    /// Returns a DeployedContract object containing the contract's ABI and deployed address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address at which the contract was deployed
    ///
    /// # Returns
    ///
    /// A DeployedContract object containing the contract's ABI and deployed address.
    pub fn deployed_at(&self, address: Address) -> DeployedContract {
        DeployedContract {
            abi: self.abi.clone(),
            address,
        }
    }

    pub fn deploy_without_constructor(&self, nonce: u128) -> LegacyTransaction {
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: None,
            value: Default::default(),
            data: self.code.clone(),
        }
    }
}

/// Compiles a Solidity contract. The `source_path` parameter specifies the directory containing all Solidity
/// source files to consider (including imports). The `contract_file` parameter must be given relative to `source_path`.
/// The `output_path` parameter specifies the directory where the compiled artifacts will be written. 
/// This function requires Docker to be installed.
///
/// # Arguments
///
/// * `source_path` - The directory containing all Solidity source files to consider (including imports)
/// * `contract_file` - The path to the Solidity contract file, relative to `source_path`
/// * `output_path` - The directory where the compiled artifacts will be written
///
/// # Panics
///
/// This function will panic if the Solidity contract cannot be compiled.
fn compile<P1, P2, P3>(source_path: P1, contract_file: P2, output_path: P3)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    P3: AsRef<Path>,
{
    let source_path = fs::canonicalize(source_path).unwrap();
    fs::create_dir_all(&output_path).unwrap();
    let output_path = fs::canonicalize(output_path).unwrap();
    let source_mount_arg = format!("{}:/contracts", source_path.to_str().unwrap());
    let output_mount_arg = format!("{}:/output", output_path.to_str().unwrap());
    let contract_arg = format!("/contracts/{}", contract_file.as_ref().to_str().unwrap());
    let output = Command::new("/usr/bin/env")
        .args([
            "docker",
            "run",
            "-v",
            &source_mount_arg,
            "-v",
            &output_mount_arg,
            "ethereum/solc:stable",
            "--allow-paths",
            "/contracts/",
            "-o",
            "/output",
            "--abi",
            "--bin",
            "--overwrite",
            &contract_arg,
        ])
        .output()
        .unwrap();
    if !output.status.success() {
        panic!(
            "Could not compile solidity contracts in docker: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }
}

impl DeployedContract {
    pub fn new(abi: Contract, address: Address) -> Self {
        Self { abi, address }
    }

    pub fn call_method_without_args(&self, method_name: &str, nonce: u128) -> LegacyTransaction {
        self.call_method_with_args(method_name, &[], nonce)
    }

    pub fn call_method_with_args(
        &self,
        method_name: &str,
        args: &[ethabi::Token],
        nonce: u128,
    ) -> LegacyTransaction {
        let data = self
        .abi
        .function(method_name)
        .unwrap()
        .encode_input(args)
        .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            to: Some(addr_to_bytes20(&self.address)),
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_deploy_transaction() -> Result<()> {
        // Create a new contract instance from the artifact
        let contract = ContractConstructor::new(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        )?;

        // Generate the deployment transaction
        let tx = contract.deploy_transaction(0, &[]);

        // Verify that the transaction data is correct
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.chain, 1313161556);
        assert_eq!(tx.gas, u64::MAX as u128);
        assert_eq!(tx.to, None);
        assert_eq!(tx.value, Default::default());
        Ok(())
    }
}
