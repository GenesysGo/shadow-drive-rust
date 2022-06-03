use solana_sdk::{
    pubkey::Pubkey
};
use async_trait::async_trait;
use byte_unit::Byte;
use models::{CreateStorageAccountResponse, ShadowDriveResult, ShdwDriveResponse};

mod client_impl;
pub mod constants;
pub mod derived_addresses;
pub mod error;
pub mod models;

pub use client_impl::*;

#[async_trait]
pub trait ShadowDriveClient {
    async fn create_storage_account(
        &self,
        name: &str,
        size: Byte,
    ) -> ShadowDriveResult<CreateStorageAccountResponse>;

    async fn request_delete_storage_account(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse>;
}
