use crate::prelude::Wei;
use crate::prelude::{Address, U256};
use crate::test_utils::{AuroraWorkspaceRunner};
use crate::test_utils::{
    self,
    erc20::{ERC20Constructor, ERC20},
    Signer
};
use anyhow::Result;
use aurora_workspace::types::{SecretKey, KeyType};
use aurora_workspace_types::output::TransactionStatus;

const INITIAL_BALANCE: u64 = 1_000_000;
const INITIAL_NONCE: u64 = 0;
const TRANSFER_AMOUNT: u64 = 67;

#[tokio::test]
async fn erc20_mint() -> Result<()> {
    let (runner, source_account, source_address, dest_address, contract) = initialize_erc20().await?;

    // Validate pre-state
    assert_eq!(
        U256::zero(),
        get_address_erc20_balance(&runner, &source_account, dest_address, &contract).await?
    ); 

    // Do mint transaction
    let mint_amount: u64 = 10;
    let outcome: TransactionStatus = runner.submit_with_signer(&source_account, 
        contract.mint(dest_address, mint_amount.into(), source_account.nonce.into())
    ).await?;
    assert!(outcome.is_ok());

    // Validate post-state
    assert_eq!(
        U256::from(mint_amount),
        get_address_erc20_balance(&runner, &source_account, dest_address, &contract).await?
    );
    Ok(())
}

async fn get_address_erc20_balance(
    runner: &AuroraWorkspaceRunner,
    signer: &Signer,
    address: Address,
    contract: &ERC20,
) -> Result<U256> {
    let balance_tx = contract.balance_of(address, signer.nonce.into());

    let counter_result = match runner.submit_with_signer(&signer, balance_tx).await? {
        TransactionStatus::Succeed(bytes) => {
            U256::from_big_endian(&bytes)
        }
        TransactionStatus::OutOfFund => panic!("Out of fund!"),
        TransactionStatus::OutOfGas => panic!("Out of gas!"),
        TransactionStatus::Revert(_bytes) => panic!("Revert! {:?}", _bytes),
        _ => panic!("Failed to execute function `counter`!"),
    };
    Ok(counter_result) 
}

async fn initialize_erc20() -> Result<(AuroraWorkspaceRunner, Signer, Address, Address, ERC20)> {
    // set up Aurora runner and accounts
    let mut runner: AuroraWorkspaceRunner = AuroraWorkspaceRunner::new().await?;
    let source_address = crate::test_utils::address_from_secret_key(&runner.signer.eth_secret_key);
    let dest_address = crate::test_utils::address_from_secret_key(&Signer::random().eth_secret_key);

    //let nonce = runner.signer.use_nonce();
    let constructor = ERC20Constructor::load();
    let contract = ERC20(runner.deploy_contract(
        &runner.signer,
        |c| {
            let legacy_tx = c.deploy("TestToken", "TEST", 0);
            ethereum_tx_sign::LegacyTransaction::from(legacy_tx)
        },
        constructor,
    ).await?);
    Ok((runner.clone(), runner.signer, source_address, dest_address, contract))
}
