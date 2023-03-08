pub mod common;
use aurora_workspace::{
    types::{ KeyType, SecretKey}, EvmContract,
};
use aurora_engine::parameters::{ SetContractDataCallArgs, PauseEthConnectorCallArgs};
use aurora_engine::fungible_token::{FungibleReferenceHash, FungibleTokenMetadata};
use serde_json::json;

use std::{str::FromStr};
use workspaces::AccountId;

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[cfg(test)]
mod tests;

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
