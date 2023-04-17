use workspaces::AccountId;
use aurora_workspace::{
    contract::{EthProverConfig}, types::KeyType,
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use aurora_workspace::InitConfig;
use workspaces::Account;
use std::io::{self, Write};


// Global constants for all tests

pub const EVM_ACCOUNT_ID: &str = "aurora.test.near";
const AURORA_LOCAL_CHAIN_ID: u64 = 1313161556;
pub const OWNER_ACCOUNT_ID: &str = "owner.test.near";
const PROVER_ACCOUNT_ID: &str = "prover.test.near";
const WASM_PATH: &str = "../bin/aurora-local.wasm";
const PRIVATE_KEY: [u8; 32] = [88u8; 32];


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
