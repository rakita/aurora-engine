use super::solidity::EthContract;
use ethereum_tx_sign::LegacyTransaction;

/// A constructor for deploying the `Random` contract to the blockchain
pub struct RandomConstructor {
    contract: EthContract,
}

/// An object for interacting with the deployed `Random` contract
pub struct Random {
    contract: EthContract,
    address: [u8; 20],
}

/// Constructor for deploying Random contract
impl RandomConstructor {
    /// Creates a new instance of RandomConstructor
    ///
    /// # Returns
    ///
    /// A RandomConstructor instance
    pub fn load() -> Self {
        let contract = EthContract::new(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        )
        .unwrap();
        Self { contract }
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
        self.contract.deploy_transaction(nonce, &[])
    }
}

/// Contract object to interact with Random contract
impl Random {
    /// Creates a new instance of Random
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the deployed Random contract
    ///
    /// # Returns
    ///
    /// A Random contract instance
    pub fn new(address: [u8; 20]) -> Self {
        let contract = EthContract::new(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        )
        .unwrap();
        Self { contract, address }
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
            to: Some(self.address),
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }
}
