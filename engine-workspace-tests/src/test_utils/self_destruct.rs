use super::{
    addr_to_bytes20,
    solidity::{self},
};
use aurora_engine_types::types::Address;
use aurora_workspace::EvmContract;
use aurora_workspace_types::output::TransactionStatus;
use ethereum_tx_sign::{LegacyTransaction, Transaction};
use anyhow::Result;

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
            chain: 1313161556,
            nonce: nonce.into(),
            gas_price: Default::default(),
            gas: u64::MAX as u128,
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

impl From<solidity::DeployedContract> for SelfDestructFactory {
    fn from(contract: solidity::DeployedContract) -> Self {
        Self { contract }
    }
}

impl SelfDestructFactory {
    pub async fn deploy(&self, evm: EvmContract, nonce: u128, private_key: [u8; 32]) -> Result<Address> {
        let data = self
            .contract
            .abi
            .function("deploy")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        let deploy_tx = LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX as u128,
            to: Some(addr_to_bytes20(&self.contract.address)),
            value: Default::default(),
            data,
        };

        let signed_deploy_tx = {
            let ecdsa = deploy_tx.ecdsa(&private_key).unwrap();
            deploy_tx.sign(&ecdsa)
        };

        let address = match evm
            .as_account()
            .submit(signed_deploy_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?
        {
            TransactionStatus::Succeed(bytes) => {
                let mut address_bytes = [0u8; 20];
                // Somehow converting address type from bytes32 low-level call leaves 12 bytes blank from start
                // e.g. [0,0,0,0,0,0,0,0,0,0,0,0,...]
                address_bytes.copy_from_slice(&bytes[12..]);
                address_bytes
            }
            _ => panic!("Failed to deploy contract!"),
        };
        
        Ok(Address::try_from_slice(&address).unwrap())
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
    pub contract: solidity::DeployedContract,
}

impl SelfDestruct {
    pub async fn counter(&self, evm: EvmContract, nonce: u128, private_key: [u8; 32]) -> Result<Option<u128>> {
        let data = self
            .contract
            .abi
            .function("counter")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        let tx = LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX as u128,
            to: Some(addr_to_bytes20(&self.contract.address)),
            value: Default::default(),
            data,
        };

        let signed_tx = {
            let ecdsa = tx.ecdsa(&private_key).unwrap();
            tx.sign(&ecdsa)
        };

        let counter_result = match evm
            .as_account()
            .submit(signed_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?
        {
            TransactionStatus::Succeed(bytes) => {
                if bytes.len() == 32 {
                    Some(u128::from_be_bytes(bytes[16..32].try_into().unwrap()))
                } else {
                    None
                }
            }
            TransactionStatus::OutOfFund => panic!("Out of fund!"),
            TransactionStatus::OutOfGas => panic!("Out of gas!"),
            TransactionStatus::Revert(_bytes) => panic!("Revert! {:?}", _bytes),
            _ => panic!("Failed to execute function `counter`!"),
        };

        Ok(counter_result)
    }

    pub async fn increase(&self, evm: EvmContract, nonce: u128, private_key: [u8; 32]) -> Result<()> {
        let data = self
            .contract
            .abi
            .function("increase")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        let tx = LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX as u128,
            to: Some(addr_to_bytes20(&self.contract.address)),
            value: Default::default(),
            data,
        };

        let signed_tx = {
            let ecdsa = tx.ecdsa(&private_key).unwrap();
            tx.sign(&ecdsa)
        };

        let _result = match evm
            .as_account()
            .submit(signed_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?
        {
            TransactionStatus::Succeed(_bytes) => {
                
            }
            _ => panic!("Increase call failed!"),
        };
        Ok(())
    }

    pub async fn finish_using_submit(&self, evm: EvmContract, nonce: u128, private_key: [u8; 32]) -> Result<()>{
        let data = self
            .contract
            .abi
            .function("finish")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        let tx = LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX as u128,
            to: Some(addr_to_bytes20(&self.contract.address)),
            value: Default::default(),
            data,
        };

        let signed_tx = {
            let ecdsa = tx.ecdsa(&private_key).unwrap();
            tx.sign(&ecdsa)
        };

        let _result = match evm
            .as_account()
            .submit(signed_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?
        {
            TransactionStatus::Succeed(_bytes) => {
                
            }
            _ => panic!("Finish call failed!"),
        };
        Ok(())
    }

}

impl From<solidity::DeployedContract> for SelfDestruct {
    fn from(contract: solidity::DeployedContract) -> Self {
        Self { contract }
    }
}
