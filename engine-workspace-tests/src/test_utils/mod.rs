use aurora_engine_types::types::Address;
use workspaces::network::Sandbox;

pub mod erc20;
pub mod random;
pub mod self_destruct;
pub mod solidity;
pub mod weth;

pub fn hex_to_vec(h: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(h.strip_prefix("0x").unwrap_or(h))
}

pub fn addr_to_bytes20(address: &Address) -> [u8; 20] {
    let mut bytes20 = [0u8; 20];
    bytes20.copy_from_slice(&address.as_bytes()[0..20]);
    bytes20
}

pub async fn deploy_evm() -> Result<(Worker<Sandbox>, EvmContract, SecretKey), Error> {
    let worker = workspaces::sandbox().await.unwrap();

    worker.fast_forward(1).await.unwrap();

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, _sk) = common::init_and_deploy_contract_with_path(&worker, WASM_PATH).await?;

    return Ok((worker, evm, _sk));
}
