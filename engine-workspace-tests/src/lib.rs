// Global dependancies for all tests
pub mod common;
use workspaces::AccountId;
use aurora_workspace::{
    contract::{EthProverConfig}, types::{SecretKey, KeyType},
};
use aurora_engine::parameters::{InitCallArgs, PauseEthConnectorCallArgs, SetContractDataCallArgs};
use aurora_engine::fungible_token::FungibleTokenMetadata;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use aurora_workspace::{EvmContract, InitConfig};
use workspaces::network::Sandbox;
use workspaces::{Worker, Account};
use std::io::{self, Write};


// Global constants for all tests

pub const EVM_ACCOUNT_ID: &str = "aurora.test.near";
const AURORA_LOCAL_CHAIN_ID: u64 = 1313161556;
pub const OWNER_ACCOUNT_ID: &str = "owner.test.near";
const PROVER_ACCOUNT_ID: &str = "prover.test.near";
const WASM_PATH: &str = "../bin/aurora-local.wasm";


#[cfg(test)]
pub mod tests;

pub mod test_utils;

pub mod prelude;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOwnerArgs {
    pub new_owner: AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Raw(pub Vec<u8>);

impl BorshSerialize for Raw {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl BorshDeserialize for Raw {
    fn deserialize(bytes: &mut &[u8]) -> io::Result<Self> {
        let res = bytes.to_vec();
        *bytes = &[];
        Ok(Self(res))
    }
}
