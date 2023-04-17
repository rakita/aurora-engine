use crate::WASM_PATH;
use crate::prelude::Wei;
use crate::prelude::{Address, U256};
use crate::test_utils::solidity::DeployedContract;
use crate::test_utils::{
    self,
    erc20::{ERC20Constructor, ERC20},
    Signer,
};
use crate::test_utils::{deploy_contract, deploy_evm, submit_with_signer, init_and_deploy_contract_with_path};
use anyhow::Result;
use aurora_workspace::types::{KeyType, SecretKey};
use aurora_workspace::{EvmAccount, EvmContract};
use aurora_workspace_types::output::TransactionStatus;
use ethereum_tx_sign::{LegacyTransaction, Transaction};
use workspaces::network::Sandbox;
use workspaces::Worker;

const INITIAL_BALANCE: u64 = 1_000_000;
const INITIAL_NONCE: u64 = 0;
const TRANSFER_AMOUNT: u64 = 67;

#[tokio::test]
async fn erc20_mint() -> Result<()> {
    let worker = workspaces::sandbox().await.unwrap();

    worker.fast_forward(1).await.unwrap();

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, signer) = init_and_deploy_contract_with_path(&worker, WASM_PATH).await?;

    worker.fast_forward(1).await.unwrap();

    let source_address = crate::test_utils::address_from_secret_key(&signer.eth_secret_key);
    let dest_address = crate::test_utils::address_from_secret_key(&Signer::random().eth_secret_key);

    let constructor = ERC20Constructor::load();
    let deploy_erc20 = constructor.deploy("TestToken", "TEST", 1);
    let signed_tx = {
        let ecdsa = deploy_erc20.ecdsa(&signer.genesis_key).unwrap();
        deploy_erc20.sign(&ecdsa)
    };
    let output = match evm
        .as_account()
        .submit(signed_tx)
        .max_gas()
        .transact()
        .await?
        .into_value()
        .into_result()?
    {
        TransactionStatus::Succeed(bytes) => {
            let mut address_bytes = [0u8; 20];
            address_bytes.copy_from_slice(&bytes);
            address_bytes
        }
        _ => panic!("Failed to deploy contract!"),
    };
    let address = Address::try_from_slice(&output).unwrap();
    let c = DeployedContract {
        abi: constructor.0.abi,
        address,
    };
    let contract = ERC20(c);
    let source_account = signer;

    // Validate pre-state
    assert_eq!(
        U256::zero(),
        get_address_erc20_balance(&worker, &evm, &source_account, dest_address, &contract).await?
    );
    //source_account.use_nonce();

    // Do mint transaction
    println!("minting {} tokens at nonce {}", 10, source_account.nonce);
    let mint_amount: u64 = 10;
    let outcome: TransactionStatus = submit_with_signer(
        &worker,
        &evm,
        &source_account,
        contract.mint(
            dest_address,
            mint_amount.into(),
            source_account.nonce.into(),
        ),
    )
    .await?;
    assert!(outcome.is_ok());

    // Validate post-state
    assert_eq!(
        U256::from(mint_amount),
        get_address_erc20_balance(&worker, &evm, &source_account, dest_address, &contract).await?
    );
    Ok(())
}

async fn get_address_erc20_balance(
    worker: &Worker<Sandbox>,
    evm: &EvmContract,
    signer: &Signer,
    address: Address,
    contract: &ERC20,
) -> Result<U256> {
    let balance_tx = contract.balance_of(address, signer.nonce.into());

    let counter_result = match submit_with_signer(worker, evm, signer, balance_tx).await? {
        TransactionStatus::Succeed(bytes) => U256::from_big_endian(&bytes),
        TransactionStatus::OutOfFund => panic!("Out of fund!"),
        TransactionStatus::OutOfGas => panic!("Out of gas!"),
        TransactionStatus::Revert(_bytes) => panic!("Revert! {:?}", _bytes),
        _ => panic!("Failed to execute function `counter`!"),
    };
    Ok(counter_result)
}

async fn initialize_erc20() -> Result<(
    Worker<Sandbox>,
    EvmContract,
    Signer,
    Address,
    Address,
    ERC20,
)> {
    // set up Aurora runner and accounts
    let (worker, evm, signer) = deploy_evm().await?;

    let source_address = crate::test_utils::address_from_secret_key(&signer.eth_secret_key);
    let dest_address = crate::test_utils::address_from_secret_key(&Signer::random().eth_secret_key);

    let constructor = ERC20Constructor::load();
    let (evm2, c) = deploy_contract(
        &worker,
        evm,
        &signer,
        |c| {
            let legacy_tx: LegacyTransaction = c.deploy("TestToken", "TEST", 0);
            legacy_tx
        },
        constructor,
    )
    .await?;
    let contract = ERC20(c);

    Ok((worker, evm2, signer, source_address, dest_address, contract))
}
