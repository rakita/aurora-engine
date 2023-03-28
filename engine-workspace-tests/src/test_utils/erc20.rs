use aurora_engine_types::U256;
use ethereum_tx_sign::LegacyTransaction;
use aurora_engine_types::types::{Address, Wei};
use crate::test_utils::solidity;
use aurora_engine_transactions::NormalizedEthTransaction;
use std::path::{Path, PathBuf};
use std::sync::Once;
use git2;

pub(crate) struct ERC20Constructor(pub solidity::ContractConstructor);

pub(crate) struct ERC20(pub solidity::DeployedContract);

impl From<ERC20Constructor> for solidity::ContractConstructor {
    fn from(c: ERC20Constructor) -> Self {
        c.0
    }
}

static DOWNLOAD_ONCE: Once = Once::new();

impl ERC20Constructor {
    pub fn load() -> Self {
        Self(solidity::ContractConstructor::compile_from_source(
            Self::download_solidity_sources(),
            Self::solidity_artifacts_path(),
            "token/ERC20/presets/ERC20PresetMinterPauser.sol",
            "ERC20PresetMinterPauser",
        ))
    }

    pub fn deploy(&self, name: &str, symbol: &str, nonce: u128) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .constructor()
            .unwrap()
            .encode_input(
                self.0.code.clone(),
                &[
                    ethabi::Token::String(name.to_string()),
                    ethabi::Token::String(symbol.to_string()),
                ],
            )
            .unwrap();
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: None,
            value: Default::default(),
            data,
        }
    }

    fn download_solidity_sources() -> PathBuf {
        let sources_dir = Path::new("target").join("openzeppelin-contracts");
        let contracts_dir = sources_dir.join("contracts");
        if contracts_dir.exists() {
            contracts_dir
        } else {
            // Contracts not already present, so download them (but only once, even
            // if multiple tests running in parallel saw `contracts_dir` does not exist).
            DOWNLOAD_ONCE.call_once(|| {
                let url = "https://github.com/OpenZeppelin/openzeppelin-contracts";
                git2::Repository::clone(url, sources_dir).unwrap();
            });
            contracts_dir
        }
    }

    fn solidity_artifacts_path() -> PathBuf {
        Path::new("target").join("solidity_build")
    }
}

impl ERC20 {
    pub fn mint(&self, recipient: Address, amount: U256, nonce: u128) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .function("mint")
            .unwrap()
            .encode_input(&[
                ethabi::Token::Address(recipient.raw()),
                ethabi::Token::Uint(amount),
            ])
            .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: Some(self.0.addr_to_bytes20()),
            value: Default::default(),
            data,
        }
    }

    pub fn transfer(&self, recipient: Address, amount: U256, nonce: u128) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .function("transfer")
            .unwrap()
            .encode_input(&[
                ethabi::Token::Address(recipient.raw()),
                ethabi::Token::Uint(amount),
            ])
            .unwrap();
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: Some(self.0.addr_to_bytes20()),
            value: Default::default(),
            data,
        }
    }

    pub fn transfer_from(
        &self,
        from: Address,
        to: Address,
        amount: U256,
        nonce: u128,
    ) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .function("transferFrom")
            .unwrap()
            .encode_input(&[
                ethabi::Token::Address(from.raw()),
                ethabi::Token::Address(to.raw()),
                ethabi::Token::Uint(amount),
            ])
            .unwrap();
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: Some(self.0.addr_to_bytes20()),
            value: Default::default(),
            data,
        }
    }

    pub fn approve(&self, spender: Address, amount: U256, nonce: u128) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .function("approve")
            .unwrap()
            .encode_input(&[
                ethabi::Token::Address(spender.raw()),
                ethabi::Token::Uint(amount),
            ])
            .unwrap();
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: Some(self.0.addr_to_bytes20()),
            value: Default::default(),
            data,
        }
    }

    pub fn balance_of(&self, address: Address, nonce: u128) -> LegacyTransaction {
        let data = self
            .0
            .abi
            .function("balanceOf")
            .unwrap()
            .encode_input(&[ethabi::Token::Address(address.raw())])
            .unwrap();
        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            gas: u64::MAX.into(),
            to: Some(self.0.addr_to_bytes20()),
            value: Default::default(),
            data,
        }
    }
}

pub(crate) fn legacy_into_normalized_tx(tx: LegacyTransaction) -> NormalizedEthTransaction {
    NormalizedEthTransaction {
        address: Default::default(),
        chain_id: None,
        nonce: tx.nonce.into(),
        gas_limit: tx.gas.into(),
        max_priority_fee_per_gas: tx.gas_price.into(),
        max_fee_per_gas: tx.gas_price.into(),
        to: Some(Address::from_array(tx.to.unwrap())),
        value: Wei::new(U256::from(tx.value)),
        data: tx.data,
        access_list: Vec::new(),
    }
}
