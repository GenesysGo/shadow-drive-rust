use reqwest::multipart::{Form, Part};
use serde_json::Value;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use super::ShadowDriveClient;
use crate::{constants::SHDW_DRIVE_ENDPOINT, error::Error, models::*};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Replace an existing file on the Shadow Drive with the given updated file.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that contains the file.
    /// * `url` - The Shadow Drive url of the file you want to replace.
    /// * `data` - The updated [`ShadowFile`](crate::models::ShadowFile).
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
    /// # let file = tokio::fs::File::open("example.png")
    /// #   .await
    /// #   .expect("failed to open file");
    /// #
    /// let edit_file_response = shdw_drive_client
    ///     .edit_file(&storage_account_key, url, file)
    ///     .await?;
    /// ```
    pub async fn edit_file(
        &self,
        storage_account_key: &Pubkey,
        data: ShadowFile,
    ) -> ShadowDriveResult<ShadowUploadResponse> {
        let message_to_sign = edit_message(storage_account_key, data.name(), &data.sha256().await?);

        let signature = self
            .wallet
            .sign_message(message_to_sign.as_bytes())
            .to_string();

        let form = Form::new()
            .part("file", data.into_form_part().await?)
            .part("signer", Part::text(self.wallet.pubkey().to_string()))
            .part("message", Part::text(signature))
            .part(
                "storage_account",
                Part::text(storage_account_key.to_string()),
            );

        let response = self
            .http_client
            .post(format!("{}/upload", SHDW_DRIVE_ENDPOINT))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await?,
            });
        }

        let response = response.json::<ShadowUploadResponse>().await?;

        Ok(response)
    }
}

fn edit_message(storage_account_key: &Pubkey, filename: &str, new_hash: &str) -> String {
    format!(
        "Shadow Drive Signed Message:\n StorageAccount: {}\nFile to edit: {}\nNew file hash: {}",
        storage_account_key, filename, new_hash
    )
}
