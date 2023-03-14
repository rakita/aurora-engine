use ethereum_tx_sign::LegacyTransaction;
use super::solidity::EthContract;
// TODO: build Random precompile suited to workspace
const DEFAULT_GAS: u64 = 1_000_000_000;

/* 
pub struct Random {
    contract: EthContract,
    address: [u8; 20],
}
impl Random {
    pub fn new(nonce: u128) -> Self {
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
*/