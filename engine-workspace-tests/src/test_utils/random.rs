use super::{
    addr_to_bytes20,
    solidity::{self, ContractConstructor, DeployedContract},
};
use aurora_engine_types::types::Address;
use ethereum_tx_sign::LegacyTransaction;

/// A constructor for deploying the `Random` contract to the blockchain
pub struct RandomConstructor(pub ContractConstructor);

/// Constructor for deploying Random contract
impl RandomConstructor {
    /// Creates a new instance of RandomConstructor
    ///
    /// # Returns
    ///
    /// A RandomConstructor instance
    pub fn load() -> Self {
        Self(solidity::ContractConstructor::compile_from_extended_json(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        ))
    }

    /// Deploys the Random contract to the blockchain
    ///
    /// # Arguments
    ///
    /// * `nonce` - A unique nonce value to ensure transaction uniqueness
    ///
    /// # Returns
    ///
    /// A LegacyTransaction containing the transaction details for submitting to NEAR Workspace
    pub fn deploy(&self, nonce: u128) -> LegacyTransaction {
        self.0.deploy_transaction(nonce, &[])
    }
}

impl From<RandomConstructor> for ContractConstructor {
    fn from(c: RandomConstructor) -> Self {
        c.0
    }
}

/// An object for interacting with the deployed `Random` contract
pub struct Random {
    contract: DeployedContract,
}

/// Contract object to interact with Random contract
impl Random {
    pub fn new (constructor: ContractConstructor, address: [u8;20]) -> Self {
        let contract = DeployedContract::new(constructor.abi, Address::from_array(address));
        Self { contract }
    }
    /// Generates a random seed using the deployed Random contract
    ///
    /// # Arguments
    ///
    /// * `nonce` - A unique nonce value to ensure transaction uniqueness
    ///
    /// # Returns
    ///
    /// A LegacyTransaction containing the transaction details for submitting to NEAR Workspace
    pub fn random_seed(&self, nonce: u128) -> LegacyTransaction {
        let data = self
            .contract
            .abi
            .function("randomSeed")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            to: Some(addr_to_bytes20(&self.contract.address)),
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }
}

impl From<solidity::DeployedContract> for Random {
    fn from(contract: solidity::DeployedContract) -> Self {
        Self { contract }
    }
}