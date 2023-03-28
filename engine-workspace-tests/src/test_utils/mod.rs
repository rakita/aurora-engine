use aurora_engine_types::types::Address;
use aurora_workspace::{types::SecretKey, EvmContract};
use anyhow::Error;
use workspaces::{network::Sandbox, Worker};
use crate::*;
use crate::WASM_PATH;

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
    let (evm, sk) = init_and_deploy_contract_with_path(&worker, WASM_PATH).await?;

    return Ok((worker, evm, sk));
}

pub struct Signer {
    pub nonce: u64,
    pub secret_key: SecretKey,
}

impl Signer {
    pub fn new(secret_key: SecretKey) -> Self {
        Self {
            nonce: 0,
            secret_key,
        }
    }

    pub fn random() -> Self {
        let sk = SecretKey::from_random(KeyType::ED25519);
        Self::new(sk)
    }

    pub fn use_nonce(&mut self) -> u64 {
        let nonce = self.nonce;
        self.nonce += 1;
        nonce
    }
}

pub async fn create_account(worker: &Worker<Sandbox>, id: &str, sk: Option<SecretKey>) -> anyhow::Result<Account> {
    let secret = sk.unwrap_or_else(|| SecretKey::from_random(KeyType::ED25519));
    let account = worker
        .create_tla(AccountId::from_str(id)?, secret)
        .await?
        .into_result()?;
    Ok(account) 
}

pub async fn init_and_deploy_contract_with_path(worker: &Worker<Sandbox>, path: &str) -> anyhow::Result<(EvmContract, SecretKey)> {
    let sk = SecretKey::from_random(KeyType::ED25519);
    let evm_account = worker
        .create_tla(AccountId::from_str(EVM_ACCOUNT_ID)?, sk.clone())
        .await?
        .into_result()?;
    let eth_prover_config = EthProverConfig::default();
    let init_config = InitConfig {
        owner_id: AccountId::from_str(OWNER_ACCOUNT_ID)?,
        prover_id: AccountId::from_str(PROVER_ACCOUNT_ID)?,
        eth_prover_config: Some(eth_prover_config),
        chain_id: AURORA_LOCAL_CHAIN_ID.into(),
    };
    let wasm = std::fs::read(path)?;
    // create contract
    let contract = EvmContract::deploy_and_init(evm_account.clone(), init_config, wasm).await?;

    Ok((contract, sk))
}

pub async fn init_and_deploy_contract_with_path_on_admin_change(worker: &Worker<Sandbox>, path: &str) -> anyhow::Result<(EvmContract, SecretKey, Account)> {
    let sk = SecretKey::from_random(KeyType::ED25519);
    let evm_account = worker
        .create_tla(AccountId::from_str(OWNER_ACCOUNT_ID)?, sk.clone())
        .await?
        .into_result()?;
    let eth_prover_config = EthProverConfig::default();
    let init_config = InitConfig {
        owner_id: AccountId::from_str(OWNER_ACCOUNT_ID)?,
        prover_id: AccountId::from_str(PROVER_ACCOUNT_ID)?,
        eth_prover_config: Some(eth_prover_config),
        chain_id: AURORA_LOCAL_CHAIN_ID.into(),
    };
    let wasm = std::fs::read(path)?;
    // create contract
    let contract = EvmContract::deploy_and_init(evm_account.clone(), init_config, wasm).await?;

    Ok((contract, sk, evm_account))
}

pub async fn init_and_deploy_contract(worker: &Worker<Sandbox>) -> anyhow::Result<EvmContract> {
    let sk = SecretKey::from_random(KeyType::ED25519);
    let evm_account = worker
        .create_tla(AccountId::from_str(EVM_ACCOUNT_ID)?, sk)
        .await?
        .into_result()?;
    let eth_prover_config = EthProverConfig::default();
    let init_config = InitConfig {
        owner_id: AccountId::from_str(OWNER_ACCOUNT_ID)?,
        prover_id: AccountId::from_str(PROVER_ACCOUNT_ID)?,
        eth_prover_config: Some(eth_prover_config),
        chain_id: AURORA_LOCAL_CHAIN_ID.into(),
    };
    let wasm = std::fs::read("./res/aurora-testnet.wasm")?;
    // create contract
    let contract = EvmContract::deploy_and_init(evm_account, init_config, wasm).await?;

    Ok(contract)
}

pub async fn init_and_deploy_sputnik(worker: &Worker<Sandbox>) -> anyhow::Result<EvmContract> {
    let sk = SecretKey::from_random(KeyType::ED25519);
    let evm_account = worker
        .create_tla(AccountId::from_str(EVM_ACCOUNT_ID)?, sk)
        .await?
        .into_result()?;
    let eth_prover_config = EthProverConfig::default();
    let init_config = InitConfig {
        owner_id: AccountId::from_str(OWNER_ACCOUNT_ID)?,
        prover_id: AccountId::from_str(PROVER_ACCOUNT_ID)?,
        eth_prover_config: Some(eth_prover_config),
        chain_id: AURORA_LOCAL_CHAIN_ID.into(),
    };
    let wasm = std::fs::read("./res/aurora-testnet.wasm")?;
    // create contract
    let contract = EvmContract::deploy_and_init(evm_account, init_config, wasm).await?;

    Ok(contract)
}