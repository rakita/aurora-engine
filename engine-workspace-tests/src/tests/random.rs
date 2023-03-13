use crate::{ common, WASM_PATH};
use aurora_engine_types::H256;
use anyhow::{Result};

#[tokio::test]
async fn test_random_number_precompile() -> Result<()> {
    let random_seed = H256::from_slice(vec![7; 32].as_slice());
    // 1. Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    worker.fast_forward(1).await?;

    // 2. deploy the Aurora EVM in sandbox with initial call to setup admin account from sender
    let (evm, _sk) = common::init_and_deploy_contract_with_path(
        &worker,
        WASM_PATH,
    )
    .await
    .unwrap();

    /* 
    let random_ctr = RandomConstructor::load();
    let nonce = signer.use_nonce();
    let random: Random = runner
        .deploy_contract(&signer.secret_key, |ctr| ctr.deploy(nonce), random_ctr)
        .into();

    let counter_value = random.random_seed(&mut runner, &mut signer);
    assert_eq!(counter_value, random_seed);
    */
    Ok(())
}
