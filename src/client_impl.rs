use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::Signer;

mod add_storage;
mod cancel_delete_file;
mod cancel_delete_storage_account;
mod claim_stake;
mod create_storage_account;
mod delete_file;
mod delete_storage_account;
mod edit_file;
mod get_storage_account;
mod list_objects;
mod make_storage_immutable;
mod reduce_storage;
mod upload_file;
mod upload_multiple_files;

pub use add_storage::*;
pub use cancel_delete_file::*;
pub use cancel_delete_storage_account::*;
pub use claim_stake::*;
pub use create_storage_account::*;
pub use delete_file::*;
pub use delete_storage_account::*;
pub use edit_file::*;
pub use get_storage_account::*;
pub use list_objects::*;
pub use make_storage_immutable::*;
pub use reduce_storage::*;
pub use upload_file::*;
pub use upload_multiple_files::*;

use crate::{
    constants::SHDW_DRIVE_ENDPOINT,
    error::Error,
    models::{FileDataResponse, ShadowDriveResult},
};

pub struct Client<T>
where
    T: Signer,
{
    wallet: T,
    rpc_client: RpcClient,
    http_client: reqwest::Client,
}

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub fn new(wallet: T, rpc_client: RpcClient) -> Self {
        Self {
            wallet,
            rpc_client,
            http_client: reqwest::Client::new(),
        }
    }

    async fn get_object_data(&self, location: &str) -> ShadowDriveResult<FileDataResponse> {
        let response = self
            .http_client
            .post(format!("{}/get-object-data", SHDW_DRIVE_ENDPOINT))
            .header("Content-Type", "application/json")
            .json(&json!({ "location": location }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await?,
            });
        }

        let response = response.json::<FileDataResponse>().await?;

        Ok(response)
    }
}
