use ethabi::{Constructor, Contract};
use ethereum_tx_sign::LegacyTransaction;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use std::error::Error;
use std::fmt;
use std::fs::{self};
use std::io::Error as IoError;
use std::path::PathBuf;

use super::hex_to_vec;

/// A struct representing an Ethereum smart contract artifact
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    /// The contract's ABI
    pub abi: Contract,
    /// The contract's bytecode
    pub bytecode: String,
}

// EthContract Errors
#[derive(Debug)]
pub enum EthContractError {
    /// An I/O error occurred
    IoError(IoError),
    /// A JSON parsing error occurred
    JsonError(JsonError),
    /// A hexadecimal decoding error occurred
    HexError(hex::FromHexError),
}

impl Error for EthContractError {}

impl fmt::Display for EthContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EthContractError::IoError(e) => write!(f, "I/O error: {}", e),
            EthContractError::JsonError(e) => write!(f, "JSON error: {}", e),
            EthContractError::HexError(e) => write!(f, "Hex decoding error: {}", e),
        }
    }
}

impl From<IoError> for EthContractError {
    fn from(error: IoError) -> Self {
        EthContractError::IoError(error)
    }
}

impl From<JsonError> for EthContractError {
    fn from(error: JsonError) -> Self {
        EthContractError::JsonError(error)
    }
}

impl From<hex::FromHexError> for EthContractError {
    fn from(error: hex::FromHexError) -> Self {
        EthContractError::HexError(error)
    }
}

/// A struct representing an Ethereum smart contract
pub struct EthContract {
    /// The contract's ABI
    pub abi: Contract,
    /// The contract's bytecode
    pub code: Vec<u8>,
}

impl EthContract {
    /// Creates a new instance of `EthContract` by reading the contract artifact from the given file path
    ///
    /// # Arguments
    ///
    /// * `artifact_path` - The file path of the contract artifact JSON file
    ///
    /// # Returns
    ///
    /// A `Result` containing the `EthContract` instance if successful, or an `EthContractError` if an error occurred
    pub fn new(artifact_path: &str) -> Result<Self, EthContractError> {
        let json_str = fs::read_to_string(PathBuf::from(artifact_path))?;
        let artifact: Artifact = serde_json::from_str(&json_str)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_deploy_transaction() -> Result<()> {
        // Create a new contract instance from the artifact
        let contract = EthContract::new(
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
