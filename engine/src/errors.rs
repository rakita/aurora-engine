pub use aurora_engine_types::parameters::engine::errors::{
    ERR_CALL_TOO_DEEP, ERR_OUT_OF_FUNDS, ERR_OUT_OF_GAS, ERR_OUT_OF_OFFSET, ERR_REVERT,
};

pub const ERR_NOT_A_JSON_TYPE: &[u8; 19] = b"ERR_NOT_A_JSON_TYPE";
pub const ERR_JSON_MISSING_VALUE: &[u8; 22] = b"ERR_JSON_MISSING_VALUE";
pub const ERR_FAILED_PARSE_U8: &[u8; 19] = b"ERR_FAILED_PARSE_U8";
pub const ERR_FAILED_PARSE_U64: &[u8; 20] = b"ERR_FAILED_PARSE_U64";
pub const ERR_FAILED_PARSE_U128: &[u8; 21] = b"ERR_FAILED_PARSE_U128";
pub const ERR_FAILED_PARSE_BOOL: &[u8; 21] = b"ERR_FAILED_PARSE_BOOL";
pub const ERR_FAILED_PARSE_STRING: &[u8; 23] = b"ERR_FAILED_PARSE_STRING";
pub const ERR_FAILED_PARSE_ARRAY: &[u8; 22] = b"ERR_FAILED_PARSE_ARRAY";
pub const ERR_EXPECTED_STRING_GOT_NUMBER: &[u8; 30] = b"ERR_EXPECTED_STRING_GOT_NUMBER";
pub const ERR_OUT_OF_RANGE_U8: &[u8; 19] = b"ERR_OUT_OF_RANGE_U8";
pub const ERR_OUT_OF_RANGE_U128: &[u8; 21] = b"ERR_OUT_OF_RANGE_U128";

pub const ERR_PROMISE_COUNT: &[u8; 17] = b"ERR_PROMISE_COUNT";
pub const ERR_REFUND_FAILURE: &[u8; 18] = b"ERR_REFUND_FAILURE";
pub const ERR_NOT_ALLOWED_TOO_EARLY: &[u8; 25] = b"ERR_NOT_ALLOWED:TOO_EARLY";
pub const ERR_PROMISE_FAILED: &[u8; 18] = b"ERR_PROMISE_FAILED";
pub const ERR_VERIFY_PROOF: &[u8; 16] = b"ERR_VERIFY_PROOF";
pub const ERR_INVALID_UPGRADE: &[u8; 19] = b"ERR_INVALID_UPGRADE";
pub const ERR_NO_UPGRADE: &[u8; 14] = b"ERR_NO_UPGRADE";
pub const ERR_NOT_ALLOWED: &[u8; 15] = b"ERR_NOT_ALLOWED";
pub const ERR_PAUSED: &[u8; 10] = b"ERR_PAUSED";
pub const ERR_RUNNING: &[u8; 11] = b"ERR_RUNNING";
pub const ERR_INVALID_BLOCK: &[u8; 17] = b"ERR_INVALID_BLOCK";

pub const ERR_SERIALIZE: &str = "ERR_SERIALIZE";
pub const ERR_PROMISE_ENCODING: &str = "ERR_PROMISE_ENCODING";
pub const ERR_ARGS: &str = "ERR_ARGS";
pub const ERR_VALUE_CONVERSION: &str = "ERR_VALUE_CONVERSION";

pub const ERR_BORSH_DESERIALIZE: &str = "ERR_BORSH_DESERIALIZE";
pub const ERR_META_TX_PARSE: &str = "ERR_META_TX_PARSE";

pub const ERR_STACK_UNDERFLOW: &[u8; 19] = b"ERR_STACK_UNDERFLOW";
pub const ERR_STACK_OVERFLOW: &[u8; 18] = b"ERR_STACK_OVERFLOW";
pub const ERR_INVALID_JUMP: &[u8; 16] = b"ERR_INVALID_JUMP";
pub const ERR_INVALID_RANGE: &[u8; 17] = b"ERR_INVALID_RANGE";
pub const ERR_DESIGNATED_INVALID: &[u8; 22] = b"ERR_DESIGNATED_INVALID";
pub const ERR_CREATE_COLLISION: &[u8; 20] = b"ERR_CREATE_COLLISION";
pub const ERR_CREATE_CONTRACT_LIMIT: &[u8; 25] = b"ERR_CREATE_CONTRACT_LIMIT";
pub const ERR_OUT_OF_FUND: &[u8; 15] = b"ERR_OUT_OF_FUND";
pub const ERR_NOT_SUPPORTED: &[u8; 17] = b"ERR_NOT_SUPPORTED";
pub const ERR_UNHANDLED_INTERRUPT: &[u8; 23] = b"ERR_UNHANDLED_INTERRUPT";
pub const ERR_INCORRECT_NONCE: &[u8; 19] = b"ERR_INCORRECT_NONCE";
pub const ERR_INVALID_CHAIN_ID: &[u8; 20] = b"ERR_INVALID_CHAIN_ID";
pub const ERR_INVALID_ECDSA_SIGNATURE: &[u8; 27] = b"ERR_INVALID_ECDSA_SIGNATURE";
pub const ERR_INTRINSIC_GAS: &[u8; 17] = b"ERR_INTRINSIC_GAS";
pub const ERR_MAX_PRIORITY_FEE_GREATER: &[u8; 28] = b"ERR_MAX_PRIORITY_FEE_GREATER";
pub const ERR_GAS_OVERFLOW: &[u8; 16] = b"ERR_GAS_OVERFLOW";
pub const ERR_BALANCE_OVERFLOW: &[u8; 20] = b"ERR_BALANCE_OVERFLOW";
pub const ERR_GAS_ETH_AMOUNT_OVERFLOW: &[u8; 27] = b"ERR_GAS_ETH_AMOUNT_OVERFLOW";
pub const ERR_PARSE_ADDRESS: &[u8; 17] = b"ERR_PARSE_ADDRESS";
pub const ERR_STATE_NOT_FOUND: &[u8; 19] = b"ERR_STATE_NOT_FOUND";
pub const ERR_STATE_CORRUPTED: &[u8; 19] = b"ERR_STATE_CORRUPTED";

pub const ERR_CONNECTOR_STORAGE_KEY_NOT_FOUND: &[u8; 35] = b"ERR_CONNECTOR_STORAGE_KEY_NOT_FOUND";
pub const ERR_FAILED_DESERIALIZE_CONNECTOR_DATA: &[u8; 37] =
    b"ERR_FAILED_DESERIALIZE_CONNECTOR_DATA";
pub const ERR_PROOF_EXIST: &[u8; 15] = b"ERR_PROOF_EXIST";
pub const ERR_WRONG_EVENT_ADDRESS: &[u8; 23] = b"ERR_WRONG_EVENT_ADDRESS";
pub const ERR_CONTRACT_INITIALIZED: &[u8; 24] = b"ERR_CONTRACT_INITIALIZED";

pub const ERR_RLP_FAILED: &[u8; 14] = b"ERR_RLP_FAILED";
pub const ERR_PARSE_DEPOSIT_EVENT: &[u8; 23] = b"ERR_PARSE_DEPOSIT_EVENT";
pub const ERR_INVALID_EVENT_MESSAGE_FORMAT: &[u8; 32] = b"ERR_INVALID_EVENT_MESSAGE_FORMAT";
pub const ERR_INVALID_SENDER: &[u8; 18] = b"ERR_INVALID_SENDER";
pub const ERR_INVALID_AMOUNT: &[u8; 18] = b"ERR_INVALID_AMOUNT";
pub const ERR_INVALID_FEE: &[u8; 15] = b"ERR_INVALID_FEE";
pub const ERR_INVALID_ON_TRANSFER_MESSAGE_FORMAT: &[u8; 38] =
    b"ERR_INVALID_ON_TRANSFER_MESSAGE_FORMAT";
pub const ERR_INVALID_ON_TRANSFER_MESSAGE_HEX: &[u8; 35] = b"ERR_INVALID_ON_TRANSFER_MESSAGE_HEX";
pub const ERR_INVALID_ON_TRANSFER_MESSAGE_DATA: &[u8; 36] = b"ERR_INVALID_ON_TRANSFER_MESSAGE_DATA";
pub const ERR_INVALID_ACCOUNT_ID: &[u8; 22] = b"ERR_INVALID_ACCOUNT_ID";
pub const ERR_OVERFLOW_NUMBER: &[u8; 19] = b"ERR_OVERFLOW_NUMBER";

pub const ERR_TOTAL_SUPPLY_OVERFLOW: &[u8; 25] = b"ERR_TOTAL_SUPPLY_OVERFLOW";
pub const ERR_NOT_ENOUGH_BALANCE: &[u8; 22] = b"ERR_NOT_ENOUGH_BALANCE";
pub const ERR_TOTAL_SUPPLY_UNDERFLOW: &[u8; 26] = b"ERR_TOTAL_SUPPLY_UNDERFLOW";
pub const ERR_ZERO_AMOUNT: &[u8; 15] = b"ERR_ZERO_AMOUNT";
pub const ERR_SENDER_EQUALS_RECEIVER: &[u8; 26] = b"ERR_SENDER_EQUALS_RECEIVER";
pub const ERR_ACCOUNT_NOT_REGISTERED: &[u8; 26] = b"ERR_ACCOUNT_NOT_REGISTERED";
pub const ERR_NO_AVAILABLE_BALANCE: &[u8; 24] = b"ERR_NO_AVAILABLE_BALANCE";
pub const ERR_ATTACHED_DEPOSIT_NOT_ENOUGH: &[u8; 31] = b"ERR_ATTACHED_DEPOSIT_NOT_ENOUGH";
pub const ERR_FAILED_UNREGISTER_ACCOUNT_POSITIVE_BALANCE: &[u8; 46] =
    b"ERR_FAILED_UNREGISTER_ACCOUNT_POSITIVE_BALANCE";
pub const ERR_SAME_OWNER: &[u8; 14] = b"ERR_SAME_OWNER";

pub const ERR_ACCOUNTS_COUNTER_OVERFLOW: &str = "ERR_ACCOUNTS_COUNTER_OVERFLOW";
