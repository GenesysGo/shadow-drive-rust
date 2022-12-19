use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use super::ShadowDriveClient;
use crate::{constants::SHDW_DRIVE_ENDPOINT, error::Error, models::*};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Marks a file for deletion from the Shadow Drive.
    /// Files marked for deletion are deleted at the end of each Solana epoch.
    /// Marking a file for deletion can be undone with `cancel_delete_file`,
    /// but this must be done before the end of the Solana epoch.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that contains the file.
    /// * `url` - The Shadow Drive url of the file you want to mark for deletion.
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
    /// # let url = String::from("https://shdw-drive.genesysgo.net/B7Qk2omAvchkePhdGovCVQuVpZHcieqPQCwFxeeBZGuT/file.txt");
    /// #
    /// let delete_file_response = shdw_drive_client
    ///     .delete_file(&storage_account_key, url)
    ///     .await?;
    /// ```
    pub async fn delete_file(
        &self,
        storage_account_key: &Pubkey,
        url: String,
    ) -> ShadowDriveResult<DeleteFileResponse> {
        let message_to_sign = delete_file_message(storage_account_key, &url);

        //Signature implements Display as a base58 encoded string
        let signature = self
            .wallet
            .sign_message(message_to_sign.as_bytes())
            .to_string();

        let body = json!({
            "signer": self.wallet.pubkey(),
            "message": signature,
            "location": url,
        });

        let response = self
            .http_client
            .post(format!("{}/delete-file", SHDW_DRIVE_ENDPOINT))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await?,
            });
        }

        let response = response.json::<DeleteFileResponse>().await?;

        Ok(response)
    }
}

fn delete_file_message(storage_account_key: &Pubkey, url: &str) -> String {
    format!(
        "Shadow Drive Signed Message:\nStorageAccount: {}\nFile to delete: {}",
        storage_account_key, url
    )
}
