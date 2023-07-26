#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]
#![cfg_attr(
    all(feature = "log", target_arch = "wasm32"),
    feature(panic_info_message)
)]
#![deny(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::unreadable_literal
)]

use aurora_engine_types::parameters::PromiseCreateArgs;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
extern crate core;

mod map;
pub mod parameters {
    pub use aurora_engine_types::parameters::connector::*;
    pub use aurora_engine_types::parameters::engine::*;
}
pub mod proof {
    pub use aurora_engine_types::parameters::connector::Proof;
}
pub mod accounting;
pub mod admin_controlled;
pub mod bloom;
#[cfg_attr(feature = "contract", allow(dead_code))]
pub mod connector;
pub mod deposit_event;
pub mod engine;
pub mod errors;
pub mod fungible_token;
pub mod hashchain;
pub mod pausables;
mod prelude;
pub mod state;
pub mod xcc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[cfg_attr(not(feature = "log"), allow(unused_variables))]
#[no_mangle]
pub unsafe fn on_panic(info: &::core::panic::PanicInfo) -> ! {
    #[cfg(feature = "log")]
    {
        use prelude::ToString;

        if let Some(msg) = info.message() {
            let msg = if let Some(log) = info.location() {
                prelude::format!("{} [{}]", msg, log)
            } else {
                msg.to_string()
            };
            prelude::sdk::panic_utf8(msg.as_bytes());
        } else if let Some(log) = info.location() {
            prelude::sdk::panic_utf8(log.to_string().as_bytes());
        }
    }

    ::core::arch::wasm32::unreachable();
}

#[cfg(target_arch = "wasm32")]
#[alloc_error_handler]
#[no_mangle]
pub unsafe fn on_alloc_error(_: core::alloc::Layout) -> ! {
    ::core::arch::wasm32::unreachable();
}

#[cfg(feature = "contract")]
mod contract {
    use ::function_name::named;
    use aurora_engine_types::parameters::WithdrawCallArgs;
    use parameters::{SetOwnerArgs, SetUpgradeDelayBlocksArgs};

    use crate::bloom::{self, Bloom};
    use crate::connector::{self, EthConnectorContract};
    use crate::engine::{self, Engine};
    use crate::hashchain::{
        self, blockchain_hashchain_error::BlockchainHashchainError, BlockchainHashchain,
    };
    use crate::parameters::{
        self, CallArgs, DeployErc20TokenArgs, FinishDepositCallArgs, FungibleTokenMetadata,
        GetErc20FromNep141CallArgs, GetStorageAtArgs, InitCallArgs, IsUsedProofCallArgs,
        NEP141FtOnTransferArgs, NewCallArgs, PauseEthConnectorCallArgs, PausePrecompilesCallArgs,
        ResolveTransferCallArgs, SetContractDataCallArgs, StartHashchainArgs,
        StorageDepositCallArgs, StorageWithdrawCallArgs, SubmitArgs, TransferCallCallArgs,
        ViewCallArgs,
    };
    #[cfg(feature = "evm_bully")]
    use crate::parameters::{BeginBlockArgs, BeginChainArgs};
    use crate::pausables::{
        Authorizer, EnginePrecompilesPauser, PausedPrecompilesChecker, PausedPrecompilesManager,
        PrecompileFlags,
    };
    use crate::prelude::account_id::AccountId;
    use crate::prelude::parameters::RefundCallArgs;
    use crate::prelude::sdk::types::{
        near_account_to_evm_address, SdkExpect, SdkProcess, SdkUnwrap,
    };
    use crate::prelude::storage::{bytes_to_key, KeyPrefix};
    use crate::prelude::{
        sdk, u256_to_arr, vec, Address, PromiseResult, ToString, Yocto, ERR_FAILED_PARSE, H256,
    };
    use crate::{errors, pausables, state};
    use aurora_engine_sdk::env::Env;
    use aurora_engine_sdk::io::{StorageIntermediate, IO};
    use aurora_engine_sdk::near_runtime::{Runtime, ViewEnv};
    use aurora_engine_sdk::promise::PromiseHandler;
    use aurora_engine_sdk::types::ExpectUtf8;
    use aurora_engine_types::borsh::{BorshDeserialize, BorshSerialize};
    use aurora_engine_types::parameters::engine::errors::ParseTypeFromJsonError;
    use aurora_engine_types::parameters::engine::{RelayerKeyArgs, RelayerKeyManagerArgs};
    use aurora_engine_types::parameters::{PromiseAction, PromiseBatchAction};

    #[cfg(feature = "integration-test")]
    use crate::prelude::NearGas;

    const CODE_KEY: &[u8; 4] = b"CODE";
    const CODE_STAGE_KEY: &[u8; 10] = b"CODE_STAGE";

    ///
    /// ADMINISTRATIVE METHODS
    ///

    /// Sets the configuration for the Engine.
    /// Should be called on deployment.
    #[no_mangle]
    #[named]
    pub extern "C" fn new() {
        let mut io = Runtime;

        if state::get_state(&io).is_ok() {
            sdk::panic_utf8(b"ERR_ALREADY_INITIALIZED");
        }

        let input = io.read_input().to_vec();
        let args = NewCallArgs::deserialize(&input).sdk_expect(errors::ERR_BORSH_DESERIALIZE);
        state::set_state(&mut io, &args.into()).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Get version of the contract.
    #[no_mangle]
    pub extern "C" fn get_version() {
        let mut io = Runtime;
        let version = option_env!("NEAR_EVM_VERSION")
            .map_or(&include_bytes!("../../VERSION")[..], str::as_bytes);
        io.return_output(version);
    }

    /// Get owner account id for this contract.
    #[no_mangle]
    pub extern "C" fn get_owner() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        io.return_output(state.owner_id.as_bytes());
    }

    /// Set owner account id for this contract.
    #[no_mangle]
    #[named]
    pub extern "C" fn set_owner() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        require_owner_only(&state, &io.predecessor_account_id());

        let input = io.read_input().to_vec();
        let args = SetOwnerArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);

        if state.owner_id == args.new_owner {
            sdk::panic_utf8(errors::ERR_SAME_OWNER);
        } else {
            state.owner_id = args.new_owner;
            state::set_state(&mut io, &state).sdk_unwrap();
            update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
        }
    }

    /// Get bridge prover id for this contract.
    #[no_mangle]
    pub extern "C" fn get_bridge_prover() {
        let mut io = Runtime;
        let connector = EthConnectorContract::init_instance(io).sdk_unwrap();
        io.return_output(connector.get_bridge_prover().as_bytes());
    }

    /// Get chain id for this contract.
    #[no_mangle]
    pub extern "C" fn get_chain_id() {
        let mut io = Runtime;
        io.return_output(&state::get_state(&io).sdk_unwrap().chain_id);
    }

    #[no_mangle]
    pub extern "C" fn get_upgrade_delay_blocks() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        io.return_output(&state.upgrade_delay_blocks.to_le_bytes());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn set_upgrade_delay_blocks() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        require_owner_only(&state, &io.predecessor_account_id());
        let input = io.read_input().to_vec();
        let args =
            SetUpgradeDelayBlocksArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        state.upgrade_delay_blocks = args.upgrade_delay_blocks;
        state::set_state(&mut io, &state).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    pub extern "C" fn get_upgrade_index() {
        let mut io = Runtime;
        let index = internal_get_upgrade_index();
        io.return_output(&index.to_le_bytes());
    }

    /// Stage new code for deployment.
    #[no_mangle]
    pub extern "C" fn stage_upgrade() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let delay_block_height = io.block_height() + state.upgrade_delay_blocks;
        require_owner_only(&state, &io.predecessor_account_id());
        io.read_input_and_store(&bytes_to_key(KeyPrefix::Config, CODE_KEY));
        io.write_storage(
            &bytes_to_key(KeyPrefix::Config, CODE_STAGE_KEY),
            &delay_block_height.to_le_bytes(),
        );
    }

    /// Deploy staged upgrade.
    #[no_mangle]
    pub extern "C" fn deploy_upgrade() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let index = internal_get_upgrade_index();
        if io.block_height() <= index {
            sdk::panic_utf8(errors::ERR_NOT_ALLOWED_TOO_EARLY);
        }
        Runtime::self_deploy(&bytes_to_key(KeyPrefix::Config, CODE_KEY));
        io.remove_storage(&bytes_to_key(KeyPrefix::Config, CODE_STAGE_KEY));
    }

    /// Called as part of the upgrade process (see `engine-sdk::self_deploy`). This function is meant
    /// to make any necessary changes to the state such that it aligns with the newly deployed
    /// code.
    #[no_mangle]
    #[allow(clippy::missing_const_for_fn)]
    pub extern "C" fn state_migration() {
        // TODO: currently we don't have migrations
    }

    /// Resumes previously [`paused`] precompiles.
    ///
    /// [`paused`]: crate::contract::pause_precompiles
    #[no_mangle]
    #[named]
    pub extern "C" fn resume_precompiles() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let predecessor_account_id = io.predecessor_account_id();

        require_owner_only(&state, &predecessor_account_id);

        let input = io.read_input().to_vec();
        let args =
            PausePrecompilesCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let flags = PrecompileFlags::from_bits_truncate(args.paused_mask);
        let mut pauser = EnginePrecompilesPauser::from_io(io);
        pauser.resume_precompiles(flags);

        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Pauses a precompile.
    #[no_mangle]
    #[named]
    pub extern "C" fn pause_precompiles() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let authorizer: pausables::EngineAuthorizer = engine::get_authorizer(&io);

        if !authorizer.is_authorized(&io.predecessor_account_id()) {
            sdk::panic_utf8(b"ERR_UNAUTHORIZED");
        }

        let input = io.read_input().to_vec();
        let args =
            PausePrecompilesCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let flags = PrecompileFlags::from_bits_truncate(args.paused_mask);
        let mut pauser = EnginePrecompilesPauser::from_io(io);
        pauser.pause_precompiles(flags);

        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Returns an unsigned integer where each 1-bit means that a precompile corresponding to that bit is paused and
    /// 0-bit means not paused.
    #[no_mangle]
    pub extern "C" fn paused_precompiles() {
        let mut io = Runtime;
        let pauser = EnginePrecompilesPauser::from_io(io);
        let data = pauser.paused().bits().to_le_bytes();
        io.return_output(&data[..]);
    }

    /// Starts the hashchain from indicated block height and block hashchain values.
    /// Resumes the contract.
    /// Requires a specific Aurora Labs account.
    /// Requires contract to be in pause state.
    /// Requires that the indicated block height is before the current block height.
    /// Assumes that no tx has been accepted after the last tx included on the indicated block hashchain.
    /// This self tx is added to the started hashchain as it changes the state of the contract.
    #[no_mangle]
    #[named]
    pub extern "C" fn start_hashchain() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();

        // *** TODO requires some Aurora Labs account
        // require_account(some_AuroraLabs_account);
        require_paused(&state);

        let input = io.read_input().to_vec();
        let args = StartHashchainArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let block_height = io.block_height();

        if args.block_height >= block_height {
            sdk::panic_utf8(errors::ERR_INVALID_BLOCK)
        }

        let mut blockchain_hashchain = BlockchainHashchain::new(
            state.chain_id,
            io.current_account_id().as_bytes().to_vec(),
            args.block_height + 1,
            args.block_hashchain,
        );

        // moves hashchain from the args state to the current state
        if block_height > blockchain_hashchain.get_current_block_height() {
            blockchain_hashchain
                .move_to_block(block_height)
                .sdk_unwrap();
        }

        hashchain::storage::set_state(&mut io, &blockchain_hashchain).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
        state.is_paused = false;
        state::set_state(&mut io, &state).sdk_unwrap()
    }

    /// Cancels the hashchain mechanism.
    /// This call will remove the hashchain storage.
    /// To resume the hashchain use `start_hashchain` method.
    #[cfg(feature = "integration-test")]
    #[no_mangle]
    pub extern "C" fn cancel_hashchain() {
        let mut io = Runtime;
        hashchain::storage::remove_state(&mut io).sdk_unwrap();
    }

    /// Sets the flag to pause the contract.
    #[no_mangle]
    #[named]
    pub extern "C" fn pause_contract() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();
        require_owner_only(&state, &io.predecessor_account_id());
        require_running(&state);
        state.is_paused = true;
        state::set_state(&mut io, &state).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &[], &[], &Bloom::default());
    }

    /// Sets the flag to resume the contract.
    #[no_mangle]
    #[named]
    pub extern "C" fn resume_contract() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();
        require_owner_only(&state, &io.predecessor_account_id());
        require_paused(&state);
        state.is_paused = false;
        state::set_state(&mut io, &state).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &[], &[], &Bloom::default());
    }

    ///
    /// MUTATIVE METHODS
    ///

    /// Deploy code into the EVM.
    #[no_mangle]
    #[named]
    pub extern "C" fn deploy_code() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input = io.read_input().to_vec();
        let current_account_id = io.current_account_id();
        let mut engine: Engine<_, _> = Engine::new(
            predecessor_address(&io.predecessor_account_id()),
            current_account_id,
            io,
            &io,
        )
        .sdk_unwrap();

        Engine::deploy_code_with_input(&mut engine, input.clone(), &mut Runtime)
            .map(|res| {
                let output = res.try_to_vec().sdk_expect(errors::ERR_SERIALIZE);
                let log_bloom = bloom::get_logs_bloom(&res.logs);
                update_hashchain(&mut io, function_name!(), &input, &output, &log_bloom);
                output
            })
            .sdk_process();
        // TODO: charge for storage
    }

    /// Call method on the EVM contract.
    #[no_mangle]
    #[named]
    pub extern "C" fn call() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input = io.read_input().to_vec();
        let args = CallArgs::deserialize(&input).sdk_expect(errors::ERR_BORSH_DESERIALIZE);
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();

        // During the XCC flow the Engine will call itself to move wNEAR
        // to the user's sub-account. We do not want this move to happen
        // if prior promises in the flow have failed.
        if current_account_id == predecessor_account_id {
            let check_promise: Result<(), &[u8]> = match io.promise_result_check() {
                Some(true) | None => Ok(()),
                Some(false) => Err(b"ERR_CALLBACK_OF_FAILED_PROMISE"),
            };
            check_promise.sdk_unwrap();
        }

        let mut engine: Engine<_, _> = Engine::new(
            predecessor_address(&predecessor_account_id),
            current_account_id,
            io,
            &io,
        )
        .sdk_unwrap();

        Engine::call_with_args(&mut engine, args, &mut Runtime)
            .map(|res| {
                let output = res.try_to_vec().sdk_expect(errors::ERR_SERIALIZE);
                let log_bloom = bloom::get_logs_bloom(&res.logs);
                update_hashchain(&mut io, function_name!(), &input, &output, &log_bloom);
                output
            })
            .sdk_process();
        // TODO: charge for storage
    }

    /// Process signed Ethereum transaction.
    /// Must match `CHAIN_ID` to make sure it's signed for given chain vs replayed from another chain.
    #[no_mangle]
    #[named]
    pub extern "C" fn submit() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let input = io.read_input().to_vec();
        let current_account_id = io.current_account_id();
        let relayer_address = predecessor_address(&io.predecessor_account_id());
        let args = SubmitArgs {
            tx_data: input.clone(),
            ..Default::default()
        };

        let result = engine::submit(
            io,
            &io,
            &args,
            state,
            current_account_id,
            relayer_address,
            &mut Runtime,
        );

        result
            .map(|res| {
                let output = res.try_to_vec().sdk_expect(errors::ERR_SERIALIZE);
                let log_bloom = bloom::get_logs_bloom(&res.logs);
                update_hashchain(&mut io, function_name!(), &input, &output, &log_bloom);
                output
            })
            .sdk_process();
    }

    /// Analog of the `submit` function, but waits for the `SubmitArgs` structure rather than
    /// the array of bytes representing the transaction.
    #[no_mangle]
    #[named]
    pub extern "C" fn submit_with_args() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let input = io.read_input().to_vec();
        let args = SubmitArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let current_account_id = io.current_account_id();
        let relayer_address = predecessor_address(&io.predecessor_account_id());

        let result = engine::submit(
            io,
            &io,
            &args,
            state,
            current_account_id,
            relayer_address,
            &mut Runtime,
        );

        result
            .map(|res| {
                let output = res.try_to_vec().sdk_expect(errors::ERR_SERIALIZE);
                let log_bloom = bloom::get_logs_bloom(&res.logs);
                update_hashchain(&mut io, function_name!(), &input, &output, &log_bloom);
                output
            })
            .sdk_process();
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn register_relayer() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input_relayer_address = io.read_input_arr20().sdk_unwrap();
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let mut engine: Engine<_, _> = Engine::new(
            predecessor_address(&predecessor_account_id),
            current_account_id,
            io,
            &io,
        )
        .sdk_unwrap();
        engine.register_relayer(
            predecessor_account_id.as_bytes(),
            Address::from_array(input_relayer_address),
        );
        update_hashchain(
            &mut io,
            function_name!(),
            &input_relayer_address,
            &[],
            &Bloom::default(),
        );
    }

    /// Updates the bytecode for user's router contracts created by the engine.
    /// These contracts are where cross-contract calls initiated by the EVM precompile
    /// will be sent from.
    #[no_mangle]
    #[named]
    pub extern "C" fn factory_update() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        require_owner_only(&state, &io.predecessor_account_id());
        let input = io.read_input().to_vec();
        let router_bytecode = crate::xcc::RouterCode::new(input.clone());
        crate::xcc::update_router_code(&mut io, &router_bytecode);
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Updates the bytecode version for the given account. This is only called as a callback
    /// when a new version of the router contract is deployed to an account.
    #[no_mangle]
    #[named]
    pub extern "C" fn factory_update_address_version() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        // The function is only set to be private, otherwise callback error will happen.
        io.assert_private_call().sdk_unwrap();
        let check_deploy: Result<(), &[u8]> = match io.promise_result_check() {
            Some(true) => Ok(()),
            Some(false) => Err(b"ERR_ROUTER_DEPLOY_FAILED"),
            None => Err(b"ERR_ROUTER_UPDATE_NOT_CALLBACK"),
        };
        check_deploy.sdk_unwrap();
        let input = io.read_input().to_vec();
        let args = crate::xcc::AddressVersionUpdateArgs::try_from_slice(&input)
            .sdk_expect(errors::ERR_SERIALIZE);
        crate::xcc::set_code_version_of_address(&mut io, &args.address, args.version);
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Sets the address for the `wNEAR` ERC-20 contract. This contract will be used by the
    /// cross-contract calls feature to have users pay for their NEAR transactions.
    #[no_mangle]
    #[named]
    pub extern "C" fn factory_set_wnear_address() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        require_owner_only(&state, &io.predecessor_account_id());
        let input_address = io.read_input_arr20().sdk_unwrap();
        crate::xcc::set_wnear_address(&mut io, &Address::from_array(input_address));
        update_hashchain(
            &mut io,
            function_name!(),
            &input_address,
            &[],
            &Bloom::default(),
        );
    }

    /// Create and/or fund an XCC sub-account directly (as opposed to having one be automatically
    /// created via the XCC precompile in the EVM). The purpose of this method is to enable
    /// XCC on engine instances where wrapped NEAR (WNEAR) is not bridged.
    #[no_mangle]
    #[named]
    pub extern "C" fn fund_xcc_sub_account() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        // This method can only be called by the owner because it allows specifying the
        // account ID of the wNEAR account. This information must be accurate for the
        // sub-account to work properly, therefore this method can only be called by
        // a trusted user.
        require_owner_only(&state, &io.predecessor_account_id());
        let input = io.read_input().to_vec();
        let args =
            crate::xcc::FundXccArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        crate::xcc::fund_xcc_sub_account(&io, &mut Runtime, &io, args).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Allow receiving NEP141 tokens to the EVM contract.
    ///
    /// This function returns the amount of tokens to return to the sender.
    /// Either all tokens are transferred and tokens are returned
    /// in case of an error, or no token is returned if the transaction was successful.
    #[no_mangle]
    #[named]
    pub extern "C" fn ft_on_transfer() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input = io.read_input().to_vec();
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let mut engine: Engine<_, _> = Engine::new(
            predecessor_address(&predecessor_account_id),
            current_account_id.clone(),
            io,
            &io,
        )
        .sdk_unwrap();

        let args: NEP141FtOnTransferArgs = serde_json::from_slice(&input)
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();

        if predecessor_account_id == current_account_id {
            EthConnectorContract::init_instance(io)
                .sdk_unwrap()
                .ft_on_transfer(&engine, &args)
                .sdk_unwrap();
        } else {
            engine.receive_erc20_tokens(
                &predecessor_account_id,
                &args,
                &current_account_id,
                &mut Runtime,
            );
        }

        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    /// Deploy ERC20 token mapped to a NEP141
    #[no_mangle]
    #[named]
    pub extern "C" fn deploy_erc20_token() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input = io.read_input().to_vec();
        // Id of the NEP141 token in Near
        let args = DeployErc20TokenArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);

        let address = engine::deploy_erc20_token(args, io, &io, &mut Runtime).sdk_unwrap();
        let address_vec = &address
            .as_bytes()
            .try_to_vec()
            .sdk_expect(errors::ERR_SERIALIZE);

        io.return_output(address_vec);
        update_hashchain(
            &mut io,
            function_name!(),
            &input,
            address_vec,
            &Bloom::default(),
        );
        // TODO: charge for storage
    }

    /// Callback invoked by exit to NEAR precompile to handle potential
    /// errors in the exit call.
    #[no_mangle]
    #[named]
    pub extern "C" fn refund_on_error() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        io.assert_private_call().sdk_unwrap();

        // This function should only be called as the callback of
        // exactly one promise.
        if io.promise_results_count() != 1 {
            sdk::panic_utf8(errors::ERR_PROMISE_COUNT);
        }

        if let Some(PromiseResult::Successful(_)) = io.promise_result(0) {
            // Promise succeeded -- nothing to do
            update_hashchain(&mut io, function_name!(), &[], &[], &Bloom::default());
        } else {
            // Exit call failed; need to refund tokens
            let input = io.read_input().to_vec();
            let args = RefundCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
            let refund_result =
                engine::refund_on_error(io, &io, state, &args, &mut Runtime).sdk_unwrap();

            if !refund_result.status.is_ok() {
                sdk::panic_utf8(errors::ERR_REFUND_FAILURE);
            }

            update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
        }
    }

    /// Sets relayer key manager.
    #[no_mangle]
    pub extern "C" fn set_key_manager() {
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();

        require_owner_only(&state, &io.predecessor_account_id());

        let key_manager =
            serde_json::from_slice::<RelayerKeyManagerArgs>(&io.read_input().to_vec())
                .map(|args| args.key_manager)
                .sdk_expect(errors::ERR_JSON_DESERIALIZE);

        if state.key_manager == key_manager {
            sdk::panic_utf8(errors::ERR_SAME_KEY_MANAGER)
        } else {
            state.key_manager = key_manager;
            state::set_state(&mut io, &state).sdk_unwrap();
        }
    }

    /// Adds a relayer function call key.
    #[no_mangle]
    pub extern "C" fn add_relayer_key() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_key_manager_only(&state, &io.predecessor_account_id());

        let public_key = serde_json::from_slice::<RelayerKeyArgs>(&io.read_input().to_vec())
            .map(|args| args.public_key)
            .sdk_expect(errors::ERR_JSON_DESERIALIZE);
        let allowance = Yocto::new(io.attached_deposit());
        sdk::log!("attached key allowance: {allowance}");

        if allowance.as_u128() < 100 {
            // TODO: Clarify the minimum amount if check is needed then change error type
            sdk::panic_utf8(errors::ERR_NOT_ALLOWED);
        }

        engine::add_function_call_key(&mut io, &public_key);

        let action = PromiseAction::AddFunctionCallKey {
            public_key,
            allowance,
            nonce: 0, // not actually used - depends on block height
            receiver_id: io.current_account_id(),
            function_names: "call,submit,submit_with_args".to_string(),
        };
        let promise = PromiseBatchAction {
            target_account_id: io.current_account_id(),
            actions: vec![action],
        };

        let promise_id = unsafe { io.promise_create_batch(&promise) };
        io.promise_return(promise_id);
    }

    /// Removes a relayer function call key.
    #[no_mangle]
    pub extern "C" fn remove_relayer_key() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_key_manager_only(&state, &io.predecessor_account_id());

        let args: RelayerKeyArgs = serde_json::from_slice(&io.read_input().to_vec())
            .sdk_expect(errors::ERR_JSON_DESERIALIZE);

        engine::remove_function_call_key(&mut io, &args.public_key).sdk_unwrap();

        let action = PromiseAction::DeleteKey {
            public_key: args.public_key,
        };
        let promise = PromiseBatchAction {
            target_account_id: io.current_account_id(),
            actions: vec![action],
        };

        let promise_id = unsafe { io.promise_create_batch(&promise) };
        io.promise_return(promise_id);
    }

    ///
    /// NONMUTATIVE METHODS
    ///
    #[no_mangle]
    pub extern "C" fn view() {
        let mut io = Runtime;
        let env = ViewEnv;
        let args: ViewCallArgs = io.read_input_borsh().sdk_unwrap();
        let current_account_id = io.current_account_id();
        let engine: Engine<_, _> =
            Engine::new(args.sender, current_account_id, io, &env).sdk_unwrap();
        let result = Engine::view_with_args(&engine, args).sdk_unwrap();
        io.return_output(&result.try_to_vec().sdk_expect(errors::ERR_SERIALIZE));
    }

    #[no_mangle]
    pub extern "C" fn get_block_hash() {
        let mut io = Runtime;
        let block_height = io.read_input_borsh().sdk_unwrap();
        let account_id = io.current_account_id();
        let chain_id = state::get_state(&io)
            .map(|state| state.chain_id)
            .sdk_unwrap();
        let block_hash =
            crate::engine::compute_block_hash(chain_id, block_height, account_id.as_bytes());
        io.return_output(block_hash.as_bytes());
    }

    #[no_mangle]
    pub extern "C" fn get_last_computed_block_hashchain() {
        let mut io = Runtime;
        let blockchain_hashchain = hashchain::storage::get_state(&io).sdk_unwrap();

        let height_and_hashchain = serde_json::to_vec(&(
            blockchain_hashchain.get_current_block_height() - 1,
            blockchain_hashchain.get_previous_block_hashchain(),
        ))
        .unwrap();

        io.return_output(&height_and_hashchain);
    }

    #[no_mangle]
    pub extern "C" fn get_code() {
        let mut io = Runtime;
        let address = io.read_input_arr20().sdk_unwrap();
        let code = engine::get_code(&io, &Address::from_array(address));
        io.return_output(&code);
    }

    #[no_mangle]
    pub extern "C" fn get_balance() {
        let mut io = Runtime;
        let address = io.read_input_arr20().sdk_unwrap();
        let balance = engine::get_balance(&io, &Address::from_array(address));
        io.return_output(&balance.to_bytes());
    }

    #[no_mangle]
    pub extern "C" fn get_nonce() {
        let mut io = Runtime;
        let address = io.read_input_arr20().sdk_unwrap();
        let nonce = engine::get_nonce(&io, &Address::from_array(address));
        io.return_output(&u256_to_arr(&nonce));
    }

    #[no_mangle]
    pub extern "C" fn get_storage_at() {
        let mut io = Runtime;
        let args: GetStorageAtArgs = io.read_input_borsh().sdk_unwrap();
        let address = args.address;
        let generation = engine::get_generation(&io, &address);
        let value = engine::get_storage(&io, &args.address, &H256(args.key), generation);
        io.return_output(&value.0);
    }

    ///
    /// BENCHMARKING METHODS
    ///
    #[cfg(feature = "evm_bully")]
    #[no_mangle]
    pub extern "C" fn begin_chain() {
        use crate::prelude::U256;
        let mut io = Runtime;
        let mut state = state::get_state(&io).sdk_unwrap();
        require_owner_only(&state, &io.predecessor_account_id());
        let args: BeginChainArgs = io.read_input_borsh().sdk_unwrap();
        state.chain_id = args.chain_id;
        state::set_state(&mut io, &state).sdk_unwrap();
        // set genesis block balances
        for account_balance in args.genesis_alloc {
            engine::set_balance(
                &mut io,
                &account_balance.address,
                &crate::prelude::Wei::new(U256::from(account_balance.balance)),
            );
        }
        // return new chain ID
        io.return_output(&state::get_state(&io).sdk_unwrap().chain_id);
    }

    #[cfg(feature = "evm_bully")]
    #[no_mangle]
    pub extern "C" fn begin_block() {
        let io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_owner_only(&state, &io.predecessor_account_id());
        let _args: BeginBlockArgs = io.read_input_borsh().sdk_unwrap();
        // TODO: https://github.com/aurora-is-near/aurora-engine/issues/2
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn new_eth_connector() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        // Only the owner can initialize the EthConnector
        let is_private = io.assert_private_call();
        if is_private.is_err() {
            require_owner_only(&state, &io.predecessor_account_id());
        }

        let input = io.read_input().to_vec();
        let args = InitCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let owner_id = io.current_account_id();

        EthConnectorContract::create_contract(io, &owner_id, args).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn set_eth_connector_contract_data() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        // Only the owner can set the EthConnector contract data
        let is_private = io.assert_private_call();
        if is_private.is_err() {
            require_owner_only(&state, &io.predecessor_account_id());
        }

        let input = io.read_input().to_vec();
        let args =
            SetContractDataCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);

        connector::set_contract_data(&mut io, args).sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn withdraw() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        io.assert_one_yocto().sdk_unwrap();
        let input = io.read_input().to_vec();
        let args = WithdrawCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let result = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .withdraw_eth_from_near(&current_account_id, &predecessor_account_id, &args)
            .sdk_unwrap();
        let output = result.try_to_vec().sdk_expect(errors::ERR_SERIALIZE);
        // We intentionally do not go through the `io` struct here because we must bypass
        // the check that prevents output that is accepted by the eth_custodian
        #[allow(clippy::as_conversions)]
        unsafe {
            exports::value_return(
                u64::try_from(output.len()).sdk_expect(errors::ERR_VALUE_CONVERSION),
                output.as_ptr() as u64,
            );
        }
        update_hashchain(
            &mut io,
            function_name!(),
            &input,
            &output,
            &Bloom::default(),
        );
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn deposit() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input_raw_proof = io.read_input().to_vec();
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let promise_args = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .deposit(
                input_raw_proof.clone(),
                current_account_id,
                predecessor_account_id,
            )
            .sdk_unwrap();
        // Safety: this call is safe because it comes from the eth-connector, not users.
        // The call is to verify the user-supplied proof for the deposit, with `finish_deposit`
        // as a callback.
        let promise_id = unsafe { io.promise_create_with_callback(&promise_args) };
        io.promise_return(promise_id);
        update_hashchain(
            &mut io,
            function_name!(),
            &input_raw_proof,
            &[],
            &Bloom::default(),
        );
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn finish_deposit() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        io.assert_private_call().sdk_unwrap();

        // Check result from proof verification call
        if io.promise_results_count() != 1 {
            sdk::panic_utf8(errors::ERR_PROMISE_COUNT);
        }
        let promise_result = match io.promise_result(0) {
            Some(PromiseResult::Successful(bytes)) => {
                bool::try_from_slice(&bytes).sdk_expect(errors::ERR_PROMISE_ENCODING)
            }
            _ => sdk::panic_utf8(errors::ERR_PROMISE_FAILED),
        };
        if !promise_result {
            sdk::panic_utf8(errors::ERR_VERIFY_PROOF);
        }

        let input = io.read_input().to_vec();
        let args = FinishDepositCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let maybe_promise_args = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .finish_deposit(
                predecessor_account_id,
                current_account_id,
                args,
                io.prepaid_gas(),
            )
            .sdk_unwrap();

        if let Some(promise_args) = maybe_promise_args {
            // Safety: this call is safe because it comes from the eth-connector, not users.
            // The call will be to the Engine's ft_transfer_call`, which is needed as part
            // of the bridge flow (if depositing ETH to an Aurora address).
            let promise_id = unsafe { io.promise_create_with_callback(&promise_args) };
            io.promise_return(promise_id);
        }

        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    pub extern "C" fn is_used_proof() {
        let mut io = Runtime;
        let args: IsUsedProofCallArgs = io.read_input_borsh().sdk_unwrap();

        let is_used_proof = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .is_used_proof(&args.proof);
        let res = is_used_proof.try_to_vec().unwrap();
        io.return_output(&res[..]);
    }

    #[no_mangle]
    pub extern "C" fn ft_total_supply() {
        let io = Runtime;
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_total_eth_supply_on_near();
    }

    #[no_mangle]
    pub extern "C" fn ft_total_eth_supply_on_near() {
        let io = Runtime;
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_total_eth_supply_on_near();
    }

    #[no_mangle]
    pub extern "C" fn ft_total_eth_supply_on_aurora() {
        let io = Runtime;
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_total_eth_supply_on_aurora();
    }

    #[no_mangle]
    pub extern "C" fn ft_balance_of() {
        let io = Runtime;
        let args: parameters::BalanceOfCallArgs = serde_json::from_slice(&io.read_input().to_vec())
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_balance_of(&args);
    }

    #[no_mangle]
    pub extern "C" fn ft_balance_of_eth() {
        let io = Runtime;
        let args: parameters::BalanceOfEthCallArgs = io.read_input().to_value().sdk_unwrap();
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_balance_of_eth_on_aurora(&args)
            .sdk_unwrap();
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn ft_transfer() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        io.assert_one_yocto().sdk_unwrap();
        let predecessor_account_id = io.predecessor_account_id();
        let input = io.read_input().to_vec();
        let args: parameters::TransferCallArgs = serde_json::from_slice(&input)
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();

        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_transfer(&predecessor_account_id, &args)
            .sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn ft_resolve_transfer() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());

        io.assert_private_call().sdk_unwrap();
        if io.promise_results_count() != 1 {
            sdk::panic_utf8(errors::ERR_PROMISE_COUNT);
        }

        let input = io.read_input().to_vec();
        let args =
            ResolveTransferCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        let promise_result = io.promise_result(0).sdk_unwrap();

        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_resolve_transfer(&args, promise_result);
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn ft_transfer_call() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        // Check is payable
        io.assert_one_yocto().sdk_unwrap();

        let input = io.read_input().to_vec();
        let args: TransferCallCallArgs = serde_json::from_slice(&input)
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();
        let current_account_id = io.current_account_id();
        let predecessor_account_id = io.predecessor_account_id();
        let promise_args = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .ft_transfer_call(
                predecessor_account_id,
                current_account_id,
                args,
                io.prepaid_gas(),
            )
            .sdk_unwrap();
        // Safety: this call is safe. It is required by the NEP-141 spec that `ft_transfer_call`
        // creates a call to another contract's `ft_on_transfer` method.
        let promise_id = unsafe { io.promise_create_with_callback(&promise_args) };
        io.promise_return(promise_id);
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn storage_deposit() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        let input = io.read_input().to_vec();
        let args: StorageDepositCallArgs = serde_json::from_slice(&input)
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();
        let predecessor_account_id = io.predecessor_account_id();
        let amount = Yocto::new(io.attached_deposit());
        let maybe_promise = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .storage_deposit(predecessor_account_id, amount, args)
            .sdk_unwrap();
        if let Some(promise) = maybe_promise {
            // Safety: This call is safe. It is only a transfer back to the user in the case
            // that they over paid for their deposit.
            unsafe { io.promise_create_batch(&promise) };
        }
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn storage_unregister() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        io.assert_one_yocto().sdk_unwrap();
        let predecessor_account_id = io.predecessor_account_id();
        let input = io.read_input().to_vec();
        let force = serde_json::from_slice::<serde_json::Value>(&input)
            .ok()
            .and_then(|args| args["force"].as_bool());
        let maybe_promise = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .storage_unregister(predecessor_account_id, force)
            .sdk_unwrap();
        if let Some(promise) = maybe_promise {
            // Safety: This call is safe. It is only a transfer back to the user for their deposit.
            unsafe { io.promise_create_batch(&promise) };
        }
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn storage_withdraw() {
        let mut io = Runtime;
        require_running(&state::get_state(&io).sdk_unwrap());
        io.assert_one_yocto().sdk_unwrap();
        let input = io.read_input().to_vec();
        let args: StorageWithdrawCallArgs = serde_json::from_slice(&input)
            .map_err(Into::<ParseTypeFromJsonError>::into)
            .sdk_unwrap();
        let predecessor_account_id = io.predecessor_account_id();
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .storage_withdraw(&predecessor_account_id, &args)
            .sdk_unwrap();
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    pub extern "C" fn storage_balance_of() {
        let io = Runtime;
        let args: parameters::StorageBalanceOfCallArgs =
            serde_json::from_slice(&io.read_input().to_vec())
                .map_err(Into::<ParseTypeFromJsonError>::into)
                .sdk_unwrap();
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .storage_balance_of(&args);
    }

    #[no_mangle]
    pub extern "C" fn get_paused_flags() {
        let mut io = Runtime;
        let paused_flags = EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .get_paused_flags();
        let data = paused_flags.try_to_vec().expect(ERR_FAILED_PARSE);
        io.return_output(&data[..]);
    }

    #[no_mangle]
    #[named]
    pub extern "C" fn set_paused_flags() {
        let mut io = Runtime;
        let state = state::get_state(&io).sdk_unwrap();
        require_running(&state);
        let is_private = io.assert_private_call();
        if is_private.is_err() {
            require_owner_only(&state, &io.predecessor_account_id());
        }
        let input = io.read_input().to_vec();
        let args =
            PauseEthConnectorCallArgs::try_from_slice(&input).sdk_expect(errors::ERR_SERIALIZE);
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .set_paused_flags(&args);
        update_hashchain(&mut io, function_name!(), &input, &[], &Bloom::default());
    }

    #[no_mangle]
    pub extern "C" fn get_accounts_counter() {
        let io = Runtime;
        EthConnectorContract::init_instance(io)
            .sdk_unwrap()
            .get_accounts_counter();
    }

    #[no_mangle]
    pub extern "C" fn get_erc20_from_nep141() {
        let mut io = Runtime;
        let args: GetErc20FromNep141CallArgs = io.read_input_borsh().sdk_unwrap();

        io.return_output(
            engine::get_erc20_from_nep141(&io, &args.nep141)
                .sdk_unwrap()
                .as_slice(),
        );
    }

    #[no_mangle]
    pub extern "C" fn get_nep141_from_erc20() {
        let mut io = Runtime;
        let erc20_address: engine::ERC20Address = io.read_input().to_vec().try_into().sdk_unwrap();
        io.return_output(
            engine::nep141_erc20_map(io)
                .lookup_right(&erc20_address)
                .sdk_expect("ERC20_NOT_FOUND")
                .as_ref(),
        );
    }

    #[no_mangle]
    pub extern "C" fn ft_metadata() {
        let mut io = Runtime;
        let metadata: FungibleTokenMetadata = connector::get_metadata(&io).unwrap_or_default();
        let bytes = serde_json::to_vec(&metadata).unwrap_or_default();
        io.return_output(&bytes);
    }

    #[cfg(feature = "integration-test")]
    #[no_mangle]
    pub extern "C" fn verify_log_entry() {
        sdk::log!("Call from verify_log_entry");
        let mut io = Runtime;
        let data = true.try_to_vec().unwrap();
        io.return_output(&data[..]);
    }

    /// Function used to create accounts for tests
    #[cfg(feature = "integration-test")]
    #[no_mangle]
    pub extern "C" fn mint_account() {
        use crate::connector::ZERO_ATTACHED_BALANCE;
        use crate::prelude::{NEP141Wei, U256};
        use evm::backend::ApplyBackend;
        const GAS_FOR_VERIFY: NearGas = NearGas::new(20_000_000_000_000);
        const GAS_FOR_FINISH: NearGas = NearGas::new(50_000_000_000_000);

        let mut io = Runtime;
        let args: ([u8; 20], u64, u64) = io.read_input_borsh().sdk_expect(errors::ERR_ARGS);
        let address = Address::from_array(args.0);
        let nonce = U256::from(args.1);
        let balance = NEP141Wei::new(u128::from(args.2));
        let current_account_id = io.current_account_id();
        let mut engine: Engine<_, _> =
            Engine::new(address, current_account_id, io, &io).sdk_unwrap();
        let state_change = evm::backend::Apply::Modify {
            address: address.raw(),
            basic: evm::backend::Basic {
                balance: U256::from(balance.as_u128()),
                nonce,
            },
            code: None,
            storage: core::iter::empty(),
            reset_storage: false,
        };
        engine.apply(core::iter::once(state_change), core::iter::empty(), false);

        // Call "finish_deposit" to mint the corresponding
        // nETH NEP-141 tokens as well
        let aurora_account_id = io.current_account_id();
        let args = crate::parameters::FinishDepositCallArgs {
            new_owner_id: aurora_account_id.clone(),
            amount: balance,
            proof_key: crate::prelude::String::new(),
            relayer_id: aurora_account_id.clone(),
            fee: 0.into(),
            msg: None,
        };
        let verify_call = aurora_engine_types::parameters::PromiseCreateArgs {
            target_account_id: aurora_account_id.clone(),
            method: crate::prelude::String::from("verify_log_entry"),
            args: crate::prelude::Vec::new(),
            attached_balance: ZERO_ATTACHED_BALANCE,
            attached_gas: GAS_FOR_VERIFY,
        };
        let finish_call = aurora_engine_types::parameters::PromiseCreateArgs {
            target_account_id: aurora_account_id,
            method: crate::prelude::String::from("finish_deposit"),
            args: args.try_to_vec().unwrap(),
            attached_balance: ZERO_ATTACHED_BALANCE,
            attached_gas: GAS_FOR_FINISH,
        };
        // Safety: this call is safe because it is only used in integration tests.
        unsafe {
            io.promise_create_with_callback(
                &aurora_engine_types::parameters::PromiseWithCallbackArgs {
                    base: verify_call,
                    callback: finish_call,
                },
            )
        };
    }

    ///
    /// Utility methods.
    ///

    fn internal_get_upgrade_index() -> u64 {
        let io = Runtime;
        match io.read_u64(&bytes_to_key(KeyPrefix::Config, CODE_STAGE_KEY)) {
            Ok(index) => index,
            Err(sdk::error::ReadU64Error::InvalidU64) => {
                sdk::panic_utf8(errors::ERR_INVALID_UPGRADE)
            }
            Err(sdk::error::ReadU64Error::MissingValue) => sdk::panic_utf8(errors::ERR_NO_UPGRADE),
        }
    }

    fn require_owner_only(state: &state::EngineState, predecessor_account_id: &AccountId) {
        if &state.owner_id != predecessor_account_id {
            sdk::panic_utf8(errors::ERR_NOT_ALLOWED);
        }
    }

    fn require_paused(state: &state::EngineState) {
        if !state.is_paused {
            sdk::panic_utf8(errors::ERR_RUNNING);
        }
    }

    fn require_running(state: &state::EngineState) {
        if state.is_paused {
            sdk::panic_utf8(errors::ERR_PAUSED);
        }
    }

    fn require_key_manager_only(state: &state::EngineState, predecessor_account_id: &AccountId) {
        let key_manager = state
            .key_manager
            .as_ref()
            .expect_utf8(errors::ERR_KEY_MANAGER_IS_NOT_SET);
        if key_manager != predecessor_account_id {
            sdk::panic_utf8(errors::ERR_NOT_ALLOWED);
        }
    }

    fn predecessor_address(predecessor_account_id: &AccountId) -> Address {
        near_account_to_evm_address(predecessor_account_id.as_bytes())
    }

    fn update_hashchain(
        io: &mut Runtime,
        method_name: &str,
        input: &[u8],
        output: &[u8],
        bloom: &Bloom,
    ) {
        let hashchain_state = hashchain::storage::get_state(io);

        if matches!(hashchain_state, Err(BlockchainHashchainError::NotFound)) {
            return;
        }

        let mut blockchain_hashchain = hashchain_state.sdk_unwrap();
        let block_height = io.block_height();

        if block_height > blockchain_hashchain.get_current_block_height() {
            blockchain_hashchain
                .move_to_block(block_height)
                .sdk_unwrap();
        }

        blockchain_hashchain
            .add_block_tx(block_height, method_name, input, output, bloom)
            .sdk_unwrap();

        hashchain::storage::set_state(io, &blockchain_hashchain).sdk_unwrap();
    }

    mod exports {
        extern "C" {
            pub(crate) fn value_return(value_len: u64, value_ptr: u64);
        }
    }
}

pub trait AuroraState {
    fn add_promise(&mut self, promise: PromiseCreateArgs);
}
