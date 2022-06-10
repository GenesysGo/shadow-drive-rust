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
//!    let solana_rpc = RpcClient::new("https://ssc-dao.genesysgo.net");
//!    let shdw_drive_client = ShadowDriveClient::new(keypair, solana_rpc);
//! ```
//!
mod client;
pub use client::*;

pub mod constants;
pub mod derived_addresses;
pub mod error;
pub mod models;
