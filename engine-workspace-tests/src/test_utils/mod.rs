use aurora_engine_types::types::Address;

pub mod random;
pub mod solidity;

pub fn hex_to_vec(h: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(h.strip_prefix("0x").unwrap_or(h))
}

pub fn addr_to_bytes20(addr: &Address) -> [u8; 20] {
    let mut bytes20 = [0u8; 20];
    address.copy_from_slice(&addr.as_bytes()[0..20]);
    bytes20
}
