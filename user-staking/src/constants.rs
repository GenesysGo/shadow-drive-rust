use anchor_lang::constant;

// Beware program idl constants!
// If they are some constant expression (but not an integer/float literal),
// the idl will store the expression as a string. For example,
// 2_u32.pow(4) is stored as "2_u32.pow(4)" and not "16", despite its type
// being stored as "u32".

#[constant]
/// This is the initial cost of storage in shades per GiB. Currently set to 1 SHDW per GB.
pub const INITIAL_STORAGE_COST: u64 = 1_073_741_824;

#[constant]
/// This is the maximum size of a storage account identifier in bytes
pub const MAX_IDENTIFIER_SIZE: usize = 64;

#[constant]
/// This is the maximum storage a single user can request in one account. 10^15 bytes = 1 PB. Currently set to 100TiB
pub const INITIAL_STORAGE_AVAILABLE: u128 = 109_951_162_777_600;

#[constant]
/// Helper constant to convert between B and GB.
pub const BYTES_PER_GIB: u32 = 1_073_741_824; // 2^30 = 1,073,741,824

#[constant]
/// This is the maximum storage a single user can request in one account. 10^12 bytes = 1TB
/// Currently set to 1TiB
pub const MAX_ACCOUNT_SIZE: u64 = 1_099_511_627_776;

#[constant]
/// Minimum account size in bytes. 10^6 B = 1 MB
/// Currently set to 1KiB
pub const MIN_ACCOUNT_SIZE: u64 = 1024;

#[constant]
/// This is the maximum filename size in bytes
pub const MAX_FILENAME_SIZE: usize = 32;

#[constant]
/// This is the size of a sha256 hash in bytes
pub const SHA256_HASH_SIZE: usize = 256 / 8;

#[constant]
/// This is the maximum size of a URL we provide users.
pub const MAX_URL_SIZE: usize = 256;

#[constant]
/// This is the time elapsed in epochs before permanent deletion off of Shadow Drive.
pub const DELETION_GRACE_PERIOD: u8 = 1;

#[constant]
/// Time required to unstake, in seconds.
pub const UNSTAKE_TIME_PERIOD: i64 = 0 * 24 * 60 * 60;

#[constant]
/// Time required to unstake, in epochs.
pub const UNSTAKE_EPOCH_PERIOD: u64 = 1;

#[constant]
/// Initial crank fee, in basis points
pub const INITIAL_CRANK_FEE_BPS: u16 = 100;

// admin1 pubkey
pub mod admin1 {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet")]
    declare_id!("E9gtcGSYWNAUGEg9MT8fHBEWeEZWRia7EafbxvBGChxd");
    #[cfg(not(feature = "mainnet"))]
    declare_id!("FRANKC3ibsaBW1o2qRuu3kspyaV4gHBuUfZ5uq9SXsqa");
}

/// SHDW pubkey
pub mod shdw {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet")]
    declare_id!("SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y");
    #[cfg(not(feature = "mainnet"))]
    declare_id!("SHDWmahkzuFwa46CpG1BF3tBHUoBTfqpypWzLRL7vNX");
    pub mod emissions_wallet {
        use anchor_lang::declare_id;
        #[cfg(feature = "mainnet")]
        declare_id!("71hcWMDnvbQ81uDRdjfM2pcX3fXpdUDcMRj1DfMbLMcy");
        #[cfg(not(feature = "mainnet"))]
        declare_id!("4CKmJQMwz4zquHdhKCURtvQSArPkiaojuK15S8CmUM9V");
    }
}
