use crate::{
    common,
    test_utils::random::{Random, RandomConstructor},
    WASM_PATH,
};
use anyhow::Result;
use aurora_engine_types::H256;
use aurora_workspace_types::output::TransactionStatus;
use ethereum_tx_sign::Transaction;

const PRIVATE_KEY: [u8; 32] = [88u8; 32];

#[tokio::test]
async fn test_random_number_precompile() -> Result<()> {
    // 1. Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    worker.fast_forward(1).await?;

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, _sk) = common::init_and_deploy_contract_with_path(&worker, WASM_PATH)
        .await
        .unwrap();

    // Get contract constructor
    let random_ctr = RandomConstructor::load();

    // Create a deploy transaction and sign it.
    let signed_deploy_tx = {
        let deploy_tx = random_ctr.deploy(0);
        let ecdsa = deploy_tx.ecdsa(&PRIVATE_KEY).unwrap();
        deploy_tx.sign(&ecdsa)
    };

    // Submit the transaction and get the ETH address.
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
            address_bytes.copy_from_slice(&bytes);
            address_bytes
        }
        _ => panic!("Failed to deploy contract!"),
    };
    let random_contract = Random::new(random_ctr.into(), address);

    // Fast forward a few blocks...
    worker.fast_forward(10).await?;

    // Create a call to the Random contract and sign it.
    let random_tx = random_contract.random_seed(1);
    let ecdsa = random_tx.ecdsa(&PRIVATE_KEY).unwrap();
    let signed_random_tx = random_tx.sign(&ecdsa);
    if let TransactionStatus::Succeed(bytes) = evm
        .as_account()
        .submit(signed_random_tx)
        .max_gas()
        .transact()
        .await?
        .into_value()
        .into_result()?
    {
        println!("RANDOM SEED: {}", hex::encode(bytes.clone()));
        let counter_value: H256 = H256::from_slice(bytes.as_slice());
        assert_eq!(counter_value.0.len(), 32);
    };

    Ok(())
}
