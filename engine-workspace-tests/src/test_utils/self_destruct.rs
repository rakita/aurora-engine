use super::{
    addr_to_bytes20,
    solidity::{self, ContractConstructor, DeployedContract},
};
use aurora_engine_types::types::Address;
use ethereum_tx_sign::LegacyTransaction;

pub struct SelfDestructFactoryConstructor(pub solidity::ContractConstructor);

impl SelfDestructFactoryConstructor {
    pub fn load() -> Self {
        Self(solidity::ContractConstructor::compile_from_extended_json(
            "../etc/eth-contracts/artifacts/contracts/test/StateTest.sol/SelfDestructFactory.json",
        ))
    }

    pub fn deploy(&self, nonce: u64) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .constructor()
            .unwrap()
            .encode_input(self.0.code.clone(), &[])
            .unwrap();

        LegacyTransaction {
            nonce: nonce.into(),
            gas_price: Default::default(),
            gas_limit: U256::from(DEFAULT_GAS),
            to: None,
            value: Default::default(),
            data,
        }
    }
}

impl From<SelfDestructFactoryConstructor> for solidity::ContractConstructor {
    fn from(c: SelfDestructFactoryConstructor) -> Self {
        c.0
    }
}

pub struct SelfDestructFactory {
    contract: solidity::DeployedContract,
}

impl SelfDestructFactory {
    pub fn deploy(&self, worker: ) -> Address {
        let data = self
            .contract
            .abi
            .function("deploy")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        let tx = LegacyTransaction {
            nonce: signer.use_nonce().into(),
            gas_price: Default::default(),
            gas_limit: U256::from(DEFAULT_GAS),
            to: Some(self.contract.address),
            value: Default::default(),
            data,
        };

        let result = runner.submit_transaction(&signer.secret_key, tx).unwrap();
        let result = test_utils::unwrap_success(result);

        Address::try_from_slice(&result[12..]).unwrap()
    }
}

pub struct SelfDestructConstructor(pub solidity::ContractConstructor);

impl SelfDestructConstructor {
    pub fn load() -> Self {
        Self(solidity::ContractConstructor::compile_from_extended_json(
            "../etc/eth-contracts/artifacts/contracts/test/StateTest.sol/SelfDestruct.json",
        ))
    }
}

pub struct SelfDestruct {
    contract: solidity::DeployedContract,
}