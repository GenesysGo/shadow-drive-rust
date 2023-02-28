use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use serde::{Deserialize, Deserializer};
#[derive(Clone, Debug, Deserialize)]
pub struct StorageAccount {
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub storage_account: Pubkey,

    /// Number of bytes of storage associated with this account
    pub reserved_bytes: u64,

    /// Bytes in use
    pub current_usage: u64,

    /// Boolean to track whether storage account (and all child File accounts) are immutable
    pub immutable: bool,

    /// Delete flag
    pub to_be_deleted: bool,

    /// Delete request epoch
    pub delete_request_epoch: u32,

    /// Primary owner of StorageAccount (immutable)
    #[serde(alias = "owner1", deserialize_with = "deserialize_pubkey")]
    pub owner_1: Pubkey,

    /// Optional owner 2
    #[serde(alias = "owner2", deserialize_with = "deserialize_pubkey")]
    pub owner_2: Pubkey,

    /// Counter at time of initialization
    pub account_counter_seed: u32,

    /// Time of storage account creation
    pub creation_time: u32,

    /// Time of storage account creation
    pub creation_epoch: u32,

    /// The last epoch through which the user paid
    pub last_fee_epoch: u32,

    /// Some unique identifier that the user provides.
    /// Serves as a seed for storage account PDA.
    pub identifier: String,
}

// Copied from shadow-drive-user-staking crate to add JSON deserialization
#[derive(Clone, Debug, Deserialize)]
pub struct StorageAccountV2 {
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub storage_account: Pubkey,

    /// Number of bytes of storage associated with this account
    pub reserved_bytes: u64,

    /// Bytes in use
    pub current_usage: u64,

    /// Boolean to track whether storage account (and all child File accounts) are immutable
    pub immutable: bool,

    /// Delete flag
    pub to_be_deleted: bool,

    /// Delete request epoch
    pub delete_request_epoch: u32,

    /// Primary owner of StorageAccount (immutable)
    #[serde(alias = "owner1", deserialize_with = "deserialize_pubkey")]
    pub owner_1: Pubkey,

    /// Counter at time of initialization
    pub account_counter_seed: u32,

    /// Time of storage account creation
    pub creation_time: u32,

    /// Time of storage account creation
    pub creation_epoch: u32,

    /// The last epoch through which the user paid
    pub last_fee_epoch: u32,

    /// Some unique identifier that the user provides.
    /// Serves as a seed for storage account PDA.
    pub identifier: String,
}

fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "version")]
pub enum StorageAcct {
    V1(StorageAccount),
    V2(StorageAccountV2),
}

impl StorageAcct {
    pub fn is_immutable(&self) -> bool {
        match self {
            StorageAcct::V1(acct) => acct.immutable,
            StorageAcct::V2(acct) => acct.immutable,
        }
    }
    pub fn storage(&self) -> u64 {
        match self {
            StorageAcct::V1(acct) => acct.reserved_bytes,
            StorageAcct::V2(acct) => acct.reserved_bytes,
        }
    }
}
