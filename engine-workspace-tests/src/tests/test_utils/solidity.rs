use ethabi::{Constructor, Contract};
use ethereum_tx_sign::LegacyTransaction;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as JsonError};
use std::error::Error;
use std::fs::{self};
use std::fmt;
use std::io::Error as IoError;
use std::path::PathBuf;

use super::hex_to_vec;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub abi: Contract,
    pub bytecode: String,
}

// EthContract Errors
#[derive(Debug)]
pub enum EthContractError {
    IoError(IoError),
    JsonError(JsonError),
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

pub struct EthContract {
    pub abi: Contract,
    pub code: Vec<u8>,
}

impl EthContract {
    pub fn new(artifact_path: &str) -> Result<Self, EthContractError> {
        let json_str = fs::read_to_string(PathBuf::from(artifact_path))?;
        let artifact: Artifact = serde_json::from_str(&json_str)?;
        let code = hex_to_vec(&artifact.bytecode)?;
        Ok(Self {
            abi: artifact.abi,
            code,
        })
    }

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
        let contract = EthContract::new("../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json")?;

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

