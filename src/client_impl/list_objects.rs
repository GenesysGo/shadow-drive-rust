use std::borrow::Cow;

use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    constants::SHDW_DRIVE_ENDPOINT,
    error::Error,
    models::{ListObjectsResponse, ShadowDriveResult},
};

use super::Client;

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn list_objects<'a, 'b>(
        &'a self,
        storage_account_key: &'b Pubkey,
    ) -> ShadowDriveResult<Vec<Cow<'_, str>>> {
        let response = self
            .http_client
            .post(format!("{}/list-objects", SHDW_DRIVE_ENDPOINT))
            .json(&json!({
              "storageAccount": storage_account_key.to_string()
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await?,
            });
        }
        response
            .json::<ListObjectsResponse>()
            .await
            .map(|response| response.keys)
            .map_err(Error::from)
    }
}
