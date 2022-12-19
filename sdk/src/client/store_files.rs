use itertools::Itertools;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use sha2::{Digest, Sha256};
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use super::ShadowDriveClient;
use crate::{constants::SHDW_DRIVE_ENDPOINT, error::Error, models::*};

fn upload_message(storage_account_key: &Pubkey, filename_hash: &str) -> String {
    format!(
        "Shadow Drive Signed Message:\nStorage Account: {}\nUpload files with hash: {}",
        storage_account_key, filename_hash
    )
}

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    pub async fn store_files(
        &self,
        storage_account_key: &Pubkey,
        data: Vec<ShadowFile>,
    ) -> ShadowDriveResult<ShadowUploadResponse> {
        let filenames = data.iter().map(ShadowFile::name).join(",");

        let mut hasher = Sha256::new();
        hasher.update(&filenames);
        let filename_hash = hasher.finalize();

        let message_to_sign = upload_message(storage_account_key, &hex::encode(filename_hash));
        //Signature implements Display as a base58 encoded string
        let signature = self
            .wallet
            .sign_message(message_to_sign.as_bytes())
            .to_string();

        let mut form = Form::new();

        for file in data {
            form = form.part("file", file.into_form_part().await?)
        }

        form = form
            .part("message", Part::text(signature))
            .part("signer", Part::text(self.wallet.pubkey().to_string()))
            .part(
                "storage_account",
                Part::text(storage_account_key.to_string()),
            )
            .part("fileNames", Part::text(filenames));

        let response = self
            .http_client
            .post(format!("{}/upload", SHDW_DRIVE_ENDPOINT))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await.unwrap_or(Value::Null),
            });
        }

        let response = response.json::<ShadowUploadResponse>().await?;

        Ok(response)
    }
}
