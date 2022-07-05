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
pub use shadow_drive_user_staking::instructions::initialize_account::StorageAccount;

pub mod constants;
pub mod derived_addresses;
pub mod error;
pub mod models;
