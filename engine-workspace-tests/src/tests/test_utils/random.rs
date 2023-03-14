use super::solidity::EthContract;
use ethereum_tx_sign::LegacyTransaction;

pub struct RandomConstructor {
    contract: EthContract,
}

pub struct Random {
    contract: EthContract,
    address: [u8; 20],
}

// Constructor for deploying Random contract
impl RandomConstructor {
    pub fn load() -> Self {
        let contract = EthContract::new(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        )
        .unwrap();
        Self { contract }
    }

    pub fn deploy(&self, nonce: u128) -> LegacyTransaction {
        self.contract.deploy_transaction(nonce, &[])
    }
}

// Contract object to interact with Random contract
impl Random {
    pub fn new(address: [u8; 20]) -> Self {
        let contract = EthContract::new(
            "../etc/eth-contracts/artifacts/contracts/test/Random.sol/Random.json",
        )
        .unwrap();
        Self { contract, address }
    }

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
