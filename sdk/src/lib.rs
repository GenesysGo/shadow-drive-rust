//! # Shadow Drive Rust
//!Rust SDK for [GenesysGo's Shadow Drive](https://shdw.genesysgo.com/shadow-infrastructure-overview/shadow-drive-overview), a decentralized storage network.
//!
//! ## Basic Usage
//!
//! ```ignore
//!    //load keypair from file
//!    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
//!
//!    //create shdw drive client
//!    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");
//! ```
//!
mod client;
pub use client::*;

pub mod constants;
pub mod derived_addresses;
pub mod error;
pub mod models;

pub use {
    // allows users to specify number of bytes
    byte_unit::Byte,
    // allows users to deserialize type
    shadow_drive_user_staking::instructions::initialize_account::StorageAccount,
    // allows users to specify rpc config
    solana_client::nonblocking::rpc_client::RpcClient,
    // allows users to specify commitment level, and use pubkeys, keypairs, and signer
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signer::{
            keypair::{read_keypair_file, Keypair},
            Signer,
        },
    },
};
