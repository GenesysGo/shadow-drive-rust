use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use cryptohelpers::sha256;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_client::rpc_client::serialize_and_encode;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use std::io::SeekFrom;
use std::str::FromStr;
use tokio::io::AsyncSeekExt;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    error::{Error, FileError},
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    /// Replace an existing file on the Shadow Drive with the given new file.
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::{Client, derived_addresses::storage_account};
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
    /// # let shdw_drive_client = Client::new(keypair, rpc_client);
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
        url: &str,
        mut data: ShdwFile,
    ) -> ShadowDriveResult<ShadowUploadResponse> {
        let file_meta = data.file.metadata().await.map_err(Error::FileSystemError)?;
        let file_size = file_meta.len();

        let selected_account = self.get_storage_account(storage_account_key).await?;

        let existing_file_data = self.get_object_data(url).await?;

        let file_owner_on_chain =
            Pubkey::from_str(&existing_file_data.file_data.owner_account_pubkey)?;

        if file_owner_on_chain != self.wallet.pubkey() {
            return Err(Error::NotFileOwner);
        }

        let file_acct = Pubkey::from_str(&existing_file_data.file_data.file_account_pubkey)?;

        let mut errors = Vec::new();
        if file_size > 1_073_741_824 {
            errors.push(FileError {
                file: data.name.clone(),
                error: String::from("Exceed the 1GB limit."),
            });
        }

        //store any info about file bytes before moving into form
        let sha256_hash = sha256::compute(&mut data.file)
            .await
            .map_err(Error::FileSystemError)?;

        //seek to front of file
        data.file
            .seek(SeekFrom::Start(0))
            .await
            .map_err(Error::FileSystemError)?;

        //construct file part and create form
        let mut file_part = Part::stream(data.file);
        if data.name.as_bytes().len() > 32 {
            errors.push(FileError {
                file: data.name.clone(),
                error: String::from("File name too long. Reduce to 32 bytes long."),
            });
        } else {
            file_part = file_part.file_name(data.name.clone());
        }

        if errors.len() > 0 {
            return Err(Error::FileValidationError(errors));
        }

        let form = Form::new().part("file", file_part);

        //construct & partial sign txn
        let accounts = shdw_drive_accounts::EditFile {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            file: file_acct,
            owner: selected_account.owner_1,
            uploader: UPLOADER,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };
        let args = shdw_drive_instructions::EditFile {
            sha256_hash: hex::encode(sha256_hash.into_bytes()),
            size: file_size,
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
