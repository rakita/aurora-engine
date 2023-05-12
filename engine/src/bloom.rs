//! Based on Parity Common Eth Bloom implementation
//! Link: <https://github.com/paritytech/parity-common/blob/master/ethbloom/src/lib.rs>
//!
//! Reimplemented here since there is a large miss match in types and dependencies.
use aurora_engine_types::parameters::engine::ResultLog;
use borsh::{BorshDeserialize, BorshSerialize};
use fixed_hash::construct_fixed_hash;
use impl_serde::impl_fixed_hash_serde;

use aurora_engine_sdk::keccak;

const BLOOM_SIZE: usize = 256;
const BLOOM_BITS: u32 = 3;

construct_fixed_hash! {
    /// Bloom hash type with 256 bytes (2048 bits) size.
    #[allow(clippy::expl_impl_clone_on_copy)]
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct Bloom(BLOOM_SIZE);
}

impl_fixed_hash_serde!(Bloom, BLOOM_SIZE);

/// Returns log2.
const fn log2(x: usize) -> u32 {
    if x <= 1 {
        return 0;
    }

    let n = x.leading_zeros();
    usize::BITS - n
}

impl Bloom {
    /// Add a new element to the bloom filter
    #[allow(clippy::cast_possible_truncation)]
    pub fn accrue(&mut self, input: &[u8]) {
        let m = self.0.len();
        let bloom_bits = m * 8;
        let mask = bloom_bits - 1;
        let bloom_bytes = (log2(bloom_bits) + 7) / 8;
        let hash = keccak(input);
        let mut ptr = 0;

        for i in 0..BLOOM_BITS {
            let _ = i;
            let mut index = 0;
            for _ in 0..bloom_bytes {
                index = (index << 8) | hash[ptr] as usize;
                ptr += 1;
            }
            index &= mask;
            self.0[m - 1 - index / 8] |= 1 << (index % 8);
        }
    }

    /// Merge two bloom filters
    pub fn accrue_bloom(&mut self, bloom: &Self) {
        for i in 0..BLOOM_SIZE {
            self.0[i] |= bloom.0[i];
        }
    }
}

#[must_use]
pub fn get_logs_bloom(logs: &[ResultLog]) -> Bloom {
    let mut logs_bloom = Bloom::default();

    for log in logs {
        logs_bloom.accrue_bloom(&get_log_bloom(log));
    }

    logs_bloom
}

#[must_use]
pub fn get_log_bloom(log: &ResultLog) -> Bloom {
    let mut log_bloom = Bloom::default();

    log_bloom.accrue(log.address.as_bytes());
    for topic in &log.topics {
        log_bloom.accrue(&topic[..]);
    }

    log_bloom
}
