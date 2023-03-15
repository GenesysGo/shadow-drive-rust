use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    constants::SHDW_DRIVE_ENDPOINT,
    error::Error,
    models::{ListObjectsResponse, ShadowDriveResult},
};

use super::ShadowDriveClient;

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Gets a list of all files associated with a storage account.
    /// The output contains all of the file names as strings.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that owns the files.
    ///
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::{ShadowDriveClient, derived_addresses::storage_account};
    /// # use solana_client::rpc_client::RpcClient;
    /// # use solana_sdk::{
    /// # pubkey::Pubkey,
    /// # signature::Keypair,
    /// # signer::{keypair::read_keypair_file, Signer},
    /// # };
    /// #
    /// # let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    /// # let user_pubkey = keypair.pubkey();
    /// # let rpc_client = RpcClient::new("https://ssc-dao.genesysgo.net");
    /// # let shdw_drive_client = ShadowDriveClient::new(keypair, rpc_client);
    /// # let (storage_account_key, _) = storage_account(&user_pubkey, 0);
    /// #
    /// let files = shdw_drive_client
    ///     .list_objects(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn list_objects(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<Vec<String>> {
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
