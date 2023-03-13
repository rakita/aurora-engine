// Global dependancies for all tests
pub mod common;
use aurora_engine::fungible_token::FungibleTokenMetadata;
use aurora_engine::parameters::{
    InitCallArgs, NewCallArgs, PauseEthConnectorCallArgs, SetContractDataCallArgs,
};
use aurora_engine_types::account_id::AccountId as EngineAccountId;
use aurora_workspace::{
    contract::EthProverConfig,
    types::{KeyType, SecretKey},
};
use aurora_workspace::{EvmContract, InitConfig};
use aurora_workspace_types::{AccountId as WorkspaceAccountId, Address};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::str::FromStr;
use workspaces::network::Sandbox;
use workspaces::AccountId;
use workspaces::{Account, Worker};

// Global constants for all tests

pub const EVM_ACCOUNT_ID: &str = "aurora.test.near";
const AURORA_LOCAL_CHAIN_ID: u64 = 1313161556;
pub const OWNER_ACCOUNT_ID: &str = "owner.test.near";
const PROVER_ACCOUNT_ID: &str = "prover.test.near";
const WASM_PATH: &str = "../../bin/aurora-local.wasm";

#[cfg(test)]
pub mod tests;

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
