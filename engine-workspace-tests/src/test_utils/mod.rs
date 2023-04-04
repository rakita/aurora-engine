use crate::prelude::{sdk, Address, Wei, H256, U256};
use crate::{WASM_PATH, OWNER_ACCOUNT_ID, PROVER_ACCOUNT_ID, AURORA_LOCAL_CHAIN_ID};
use aurora_workspace::contract::EthProverConfig;
use aurora_workspace::types::KeyType;
use aurora_workspace::{types::SecretKey, EvmContract};
use aurora_workspace_types::AccountId;
use aurora_workspace_types::output::TransactionStatus;
use ethereum_tx_sign::{LegacyTransaction, Transaction};
use libsecp256k1::PublicKey;
use super::*;
use workspaces::{network::Sandbox, Worker};
use rand::RngCore;

use self::solidity::{ContractConstructor, DeployedContract};
pub mod erc20;
pub mod random;
pub mod self_destruct;
pub mod solidity;
pub mod weth;


pub(crate) fn address_from_secret_key(sk: &libsecp256k1::SecretKey) -> Address {
    let pk = PublicKey::from_secret_key(&sk);
    let hash = sdk::keccak(&pk.serialize()[1..]);
    Address::try_from_slice(&hash[12..]).unwrap()
}

pub(crate) fn hex_to_vec(h: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(h.strip_prefix("0x").unwrap_or(h))
}

pub(crate) fn addr_to_bytes20(address: &Address) -> [u8; 20] {
    let mut bytes20 = [0u8; 20];
    bytes20.copy_from_slice(&address.as_bytes()[0..20]);
    bytes20
}


#[derive(Debug, Clone)]
pub struct Signer {
    pub nonce: u64,
    pub private_key: [u8; 32],
    pub near_secret_key: SecretKey,
    pub eth_secret_key: libsecp256k1::SecretKey,
}

impl Signer {
    pub fn new(private_key: [u8; 32]) -> Self {
        let near_secret_key = SecretKey::from_seed(KeyType::ED25519, hex::encode(&private_key).as_str());
        let eth_secret_key = libsecp256k1::SecretKey::parse(&private_key).unwrap();
        Self {
            nonce: 0,
            private_key,
            near_secret_key,
            eth_secret_key,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut private_key = [0; 32];
        rng.fill_bytes(&mut private_key);
        Self::new(private_key)
    }

    pub fn use_nonce(&mut self) -> u64 {
        let nonce = self.nonce;
        self.nonce += 1;
        nonce
    }
}

pub(crate) async fn create_account(
    worker: &Worker<Sandbox>,
    id: &str,
    sk: Option<SecretKey>,
) -> anyhow::Result<Account> {
    let secret = sk.unwrap_or_else(|| SecretKey::from_random(KeyType::ED25519));
    let account = worker
        .create_tla(AccountId::from_str(id)?, secret)
        .await?
        .into_result()?;
    Ok(account)
}

pub(crate) async fn init_and_deploy_contract_with_path(
    worker: &Worker<Sandbox>,
    path: &str,
) -> anyhow::Result<(EvmContract, Signer)> {
    let signer = Signer::random();
    let evm_account = worker
        .create_tla(
            AccountId::from_str(EVM_ACCOUNT_ID)?,
            signer.near_secret_key.clone(),
        )
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

    Ok((contract, signer))
}

pub(crate) async fn init_and_deploy_contract_with_path_on_admin_change(
    worker: &Worker<Sandbox>,
    path: &str,
) -> anyhow::Result<(EvmContract, SecretKey, Account)> {
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

pub(crate) async fn init_and_deploy_contract(
    worker: &Worker<Sandbox>,
) -> anyhow::Result<EvmContract> {
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

pub(crate) async fn init_and_deploy_sputnik(
    worker: &Worker<Sandbox>,
) -> anyhow::Result<EvmContract> {
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

pub async fn submit_with_signer(
    evm: EvmContract,
    signer: &mut Signer,
    tx: LegacyTransaction,
) -> Result<TransactionStatus, anyhow::Error> {
    let signed_tx = {
        let ecdsa = tx.ecdsa(&signer.private_key).unwrap();
        tx.sign(&ecdsa)
    };

    let result = evm
        .as_account()
        .submit(signed_tx)
        .max_gas()
        .transact()
        .await?
        .into_value()
        .into_result()?;

    Ok(result)
}


#[derive(Debug, Clone)]
pub struct AuroraWorkspaceRunner {
    pub worker: Worker<Sandbox>,
    pub evm: EvmContract,
    pub signer: Signer,
}

impl AuroraWorkspaceRunner {
    pub async fn new() -> anyhow::Result<Self> {
        let worker = workspaces::sandbox().await.unwrap();

        worker.fast_forward(1).await.unwrap();

        // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
        let (evm, signer) = init_and_deploy_contract_with_path(&worker, WASM_PATH).await?;

        worker.fast_forward(1).await.unwrap();

        return Ok(AuroraWorkspaceRunner {
            worker,
            evm,
            signer,
        });
    }

    pub async fn submit_with_signer(&self, signer: &Signer, tx: LegacyTransaction) -> Result<TransactionStatus, anyhow::Error> {
        let signed_tx = {
            let ecdsa = tx.ecdsa(&signer.private_key).unwrap();
            tx.sign(&ecdsa)
        };
    
        let result = self.evm
            .as_account()
            .submit(signed_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?;
    
        Ok(result)
    }

    pub async fn create_account(
        worker: &Worker<Sandbox>,
        id: &str,
        sk: Option<SecretKey>,
    ) -> anyhow::Result<Account> {
        let secret = sk.unwrap_or_else(|| SecretKey::from_random(KeyType::ED25519));
        let account = worker
            .create_tla(AccountId::from_str(id)?, secret)
            .await?
            .into_result()?;
        Ok(account)
    }

    pub async fn deploy_contract<F: FnOnce(&T) -> LegacyTransaction, T: Into<ContractConstructor>>(
        &self,
        account: &Signer,
        constructor_tx: F,
        contract_constructor: T,
    ) -> anyhow::Result<DeployedContract, anyhow::Error> {
        let tx = constructor_tx(&contract_constructor);
        println!("nonce: {:?} {:?} {}", tx.nonce, account.nonce, tx.nonce == account.nonce);
        let signed_tx = {
            let ecdsa = tx.ecdsa(&account.private_key).unwrap();
            tx.sign(&ecdsa)
        };
        println!("Deploying contract with account: {:?}", account);
        let output = match self.evm
            .as_account()
            .submit(signed_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()? {
                TransactionStatus::Succeed(bytes) => {
                    let mut address_bytes = [0u8; 20];
                    address_bytes.copy_from_slice(&bytes);
                    address_bytes
                }
                _ => panic!("Failed to deploy contract!"),
            };
        let address =  Address::try_from_slice(&output).unwrap();
        let contract_constructor: ContractConstructor = contract_constructor.into();
        Ok(DeployedContract {
            abi: contract_constructor.abi,
            address,
        })
    }
}
