use crate::test_utils::init_and_deploy_contract_with_path;
use crate::test_utils::self_destruct::{
    SelfDestruct, SelfDestructConstructor, SelfDestructFactory, SelfDestructFactoryConstructor,
};
use crate::{WASM_PATH};
use anyhow::Result;
use aurora_engine_types::types::Address;
use aurora_workspace_types::output::TransactionStatus;
use ethereum_tx_sign::Transaction;

const PRIVATE_KEY: [u8; 32] = [88u8; 32];

/// Check that account state should be properly removed after calling selfdestruct
#[tokio::test]
async fn test_self_destruct_reset_state() -> Result<()> {
    // 1. Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    worker.fast_forward(1).await?;

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, _sk) = init_and_deploy_contract_with_path(&worker, WASM_PATH)
        .await
        .unwrap();

    let sd_factory_ctr = SelfDestructFactoryConstructor::load();
    // Create a deploy transaction and sign it.
    let signed_deploy_tx = {
        let deploy_tx = sd_factory_ctr.deploy(0);
        let ecdsa = deploy_tx.ecdsa(&PRIVATE_KEY).unwrap();
        deploy_tx.sign(&ecdsa)
    };

    // Submit the transaction and get the ETH address.
    let sd_factory_contract_addr = match evm
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

    println!(
        "sd_factory_contract_addr: {:?}",
        Address::try_from_slice(&sd_factory_contract_addr).unwrap()
    );

    worker.fast_forward(1).await?;

    // deploy sd from sd factory
    let sd_factory: SelfDestructFactory = SelfDestructFactoryConstructor::load()
        .0
        .deployed_at(Address::try_from_slice(&sd_factory_contract_addr).unwrap())
        .into();

    let sd_contract_addr = sd_factory.deploy(evm.clone(), 1, PRIVATE_KEY).await?;

    println!("sd_contract_addr: {:?}", sd_contract_addr);

    let sd: SelfDestruct = SelfDestructConstructor::load()
        .0
        .deployed_at(sd_contract_addr)
        .into();

    let counter_value = sd.counter(evm.clone(), 2, PRIVATE_KEY).await?;
    assert_eq!(counter_value, Some(0));

    sd.increase(evm.clone(), 3, PRIVATE_KEY).await?;

    let counter_value = sd.counter(evm.clone(), 4, PRIVATE_KEY).await?;
    assert_eq!(counter_value, Some(1));
    sd.finish_using_submit(evm.clone(), 5, PRIVATE_KEY).await?;
    let counter_value = sd.counter(evm.clone(), 6, PRIVATE_KEY).await?;
    assert!(counter_value.is_none());

    let sd_contract_addr1 = sd_factory.deploy(evm.clone(), 7, PRIVATE_KEY).await?;
    assert_eq!(sd_contract_addr, sd_contract_addr1);

    let counter_value = sd.counter(evm, 8, PRIVATE_KEY).await?;
    assert_eq!(counter_value, Some(0));

    Ok(())
}

#[tokio::test]
async fn test_self_destruct_with_submit() -> Result<()> {
    // 1. Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    worker.fast_forward(1).await?;

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, _sk) = init_and_deploy_contract_with_path(&worker, WASM_PATH)
        .await
        .unwrap();

    let sd_factory_ctr = SelfDestructFactoryConstructor::load();
    // Create a deploy transaction and sign it.
    let signed_deploy_tx = {
        let deploy_tx = sd_factory_ctr.deploy(0);
        let ecdsa = deploy_tx.ecdsa(&PRIVATE_KEY).unwrap();
        deploy_tx.sign(&ecdsa)
    };

    // Submit the transaction and get the ETH address.
    let sd_factory_contract_addr = match evm
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

    worker.fast_forward(1).await?;

    // deploy sd from sd factory
    let sd_factory: SelfDestructFactory = SelfDestructFactoryConstructor::load()
        .0
        .deployed_at(Address::try_from_slice(&sd_factory_contract_addr).unwrap())
        .into();

    let sd_contract_addr = sd_factory.deploy(evm.clone(), 1, PRIVATE_KEY).await?;

    let sd: SelfDestruct = SelfDestructConstructor::load()
        .0
        .deployed_at(sd_contract_addr)
        .into();

    sd.finish_using_submit(evm.clone(), 2, PRIVATE_KEY).await?;
    
    Ok(())
}
