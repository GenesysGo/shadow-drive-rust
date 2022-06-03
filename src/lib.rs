use async_trait::async_trait;
use byte_unit::Byte;
use models::{CreateStorageAccountResponse, ShadowDriveResult};

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
}
