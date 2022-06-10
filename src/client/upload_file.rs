use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_client::rpc_client::serialize_and_encode;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    derived_addresses,
    error::Error,
    models::*,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
{
    /// Uploads a [`ShadowFile`] to the Shadow Drive, using the specified [`StorageAccount`].
    /// * `storage_account_key` - The public key of the [`StorageAccount`] that will hold the file.
    /// * `data` - The [`ShadowFile`] to be uploaded.
    /// 
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::{ShadowDriveClient, derived_addresses::storage_account, models::ShadowFile};
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
    /// # let file = ShadowFile::file(String::from("example.png"), "example.png")
    /// #
    /// let upload_file_response = shdw_drive_client
    ///     .upload_file(&storage_account_key, file)
    ///     .await?;
    /// ````
    pub async fn upload_file<'a>(
        &self,
        storage_account_key: &Pubkey,
        data: ShadowFile,
    ) -> ShadowDriveResult<ShadowUploadResponse> {
        let upload_data = data
            .prepare_upload(storage_account_key)
            .await
            .map_err(Error::FileValidationError)?;

        let wallet_pubkey = self.wallet.pubkey();
        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);

        let selected_account = self.get_storage_account(storage_account_key).await?;

        let form = Form::new().part("file", upload_data.to_form_part().await?);

        //construct & partial sign txn
        let file_seed = selected_account.init_counter;
        let (file_acct, _) = derived_addresses::file_account(&storage_account_key, file_seed);

        let accounts = shdw_drive_accounts::StoreFile {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            user_info,
            file: file_acct,
            owner: selected_account.owner_1,
            uploader: UPLOADER,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };
        let args = shdw_drive_instructions::StoreFile {
            filename: String::from(upload_data.file.name),
            sha256_hash: hex::encode(upload_data.sha256_hash.into_bytes()),
            size: upload_data.size,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&self.wallet.pubkey()));
        txn.try_partial_sign(&[&self.wallet], self.rpc_client.get_latest_blockhash()?)?;

        //base64 encode txn and add to form
        let txn_encoded = serialize_and_encode(&txn, UiTransactionEncoding::Base64)?;

        let form = form.part("transaction", Part::text(txn_encoded));

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
