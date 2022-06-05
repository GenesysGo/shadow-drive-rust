use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use cryptohelpers::sha256;
use futures::future::join_all;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_client::rpc_client::serialize_and_encode;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use tokio::fs::File;
use std::collections::HashSet;
use std::fs::Metadata;
use std::io::SeekFrom;
use std::time::Duration;
use tokio::io::AsyncSeekExt;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    derived_addresses,
    error::{Error, FileError},
    models::*,
};

#[derive(Debug)]
struct UploadingData {
    name: String,
    size: u64,
    sha256_hash: sha256::Sha256Hash,
    url: String,
    file: File,
}

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn upload_multiple_files(
        &self,
        storage_account_key: &Pubkey,
        data: Vec<ShdwFile>,
    ) -> ShadowDriveResult<Vec<ShadowBatchUploadResponse>> {
        let wallet_pubkey = self.wallet.pubkey();
        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);
        let selected_account = self.get_storage_account(storage_account_key).await?;
        
        //collect upload data for each file
        let validation_futures = data
            .into_iter()
            .map(|shdw_file| async move {
              self.valide_file(shdw_file, storage_account_key).await
            })
            .collect::<Vec<_>>();

        let file_data = join_all(validation_futures).await;

        let (succeeded_files, errored_files): (Vec<_>, Vec<_>) =
            file_data.into_iter().partition(Result::is_ok);
        //it's safe to unwrap after the above partition
        let errored_files: Vec<Vec<FileError>> =
            errored_files.into_iter().map(Result::unwrap_err).collect();
        if errored_files.len() > 0 {
            return Err(Error::FileValidationError(
                errored_files.into_iter().flatten().collect(),
            ));
        }
        let succeeded_files = succeeded_files.into_iter().map(Result::unwrap);


        //filter out any existing files
        let all_objects:HashSet<String> = self.list_objects(&storage_account_key).await?.into_iter().collect();
        let (to_upload, existing_uploads): (Vec<_>, Vec<_>) = succeeded_files.into_iter().partition(|file| {
            !all_objects.contains(&file.name)
        } );

        //pre-fill results w/ existing files
        let mut upload_results = existing_uploads.into_iter().map(|file| {
          ShadowBatchUploadResponse {
            file_name: file.name,
            status: BatchUploadStatus::AlreadyExists,
            location: Some(file.url),
            transaction_signature: None,
        }
        }).collect::<Vec<_>>();

        let mut chunks = Vec::default();
        let mut current_chunk : Vec<UploadingData>= Vec::default();
        let mut name_buffer = 0;

        for file_data in to_upload {
            if current_chunk.is_empty() {
                name_buffer += file_data.name.as_bytes().len();
                current_chunk.push(file_data);
                continue;
            }

            //if the current chunk has 5 or less
            if current_chunk.len() < 5 && 
            //our current name buffer is under the limit 
            name_buffer < 154 &&
            //the name buffer will be under size with the new file
            name_buffer + file_data.name.as_bytes().len() < 154
            {
                //add to current chunk
                name_buffer += file_data.name.as_bytes().len();
                current_chunk.push(file_data);
            } else {
              //create new chunk and clear name buffer
              chunks.push(current_chunk);
              current_chunk = Vec::default();
              name_buffer = 0;
            }
        }
        //if the final chunk has something, push it to chunks
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        //confirm file seed before sending
        let mut new_file_seed = selected_account.init_counter;

        //send each chunk to shdw drive
        for chunk in chunks {
        new_file_seed = self.confirm_storage_account_seed(new_file_seed, storage_account_key).await?;

        let mut num_retries = 0;
        loop {
          match self.send_chunk(storage_account_key, user_info, &mut new_file_seed, &chunk).await {
            Ok(response) => {
              upload_results.extend(response.into_iter());
            }
            Err(error) => {
              tracing::error!(retries = num_retries, ?error, "error uploading batch to shdw drive");
              num_retries += 1;
              //after 5 attempts bail on the chunk
              if num_retries == 5 {
                //reset file seed
                new_file_seed = self.confirm_storage_account_seed(selected_account.init_counter, storage_account_key).await?;

                //save failed entries
                let failed = chunk.into_iter().map(|file| {
                  ShadowBatchUploadResponse {
                    file_name: file.name,
                    status: BatchUploadStatus::Error(format!("{:?}", error)),
                    location: None,
                    transaction_signature: None,
                  }
                });
                upload_results.extend(failed);
                //break chunk retry loop to move to next
                break
              }
            }
          }
        }
      }

      Ok(upload_results)
    }

    async fn valide_file(&self, mut shdw_file: ShdwFile, storage_account_key: &Pubkey) -> Result<UploadingData, Vec<FileError>> {
      let mut errors = Vec::new();
      let file_meta: Metadata;
      match shdw_file.file.metadata().await {
          Ok(meta) => file_meta = meta,
          Err(err) => {
              errors.push(FileError {
                  file: shdw_file.name.clone(),
                  error: format!("error opening file metadata: {:?}", err),
              });
              return Err(errors);
          }
      }
      let file_size = file_meta.len();
      if file_size > 1_073_741_824 {
          errors.push(FileError {
              file: shdw_file.name.clone(),
              error: String::from("Exceed the 1GB limit."),
          });
      }

      //this may need to be url encoded
      //should ShdwFile.name not be an option?
      let url = format!(
          "https://shdw-drive.genesysgo.net/{}/{}",
          storage_account_key.to_string(),
          &shdw_file.name.clone().unwrap_or_default()
      );

      //store any info about file bytes before moving into form
      let sha256_hash = match sha256::compute(&mut shdw_file.file).await {
          Ok(hash) => hash,
          Err(err) => {
              errors.push(FileError {
                  file: shdw_file.name.clone(),
                  error: format!("error hashing file: {:?}", err),
              });
              return Err(errors);
          }
      };

      //construct file part and create form
      if let Some(name) = shdw_file.name.as_ref() {
          if name.as_bytes().len() > 32 {
              errors.push(FileError {
                  file: Some(name.to_string()),
                  error: String::from("Exceed the 1GB limit."),
              });
          }
      }

      if errors.len() > 0 {
          return Err(errors);
      }

      Ok(UploadingData {
          name: shdw_file.name.unwrap_or_default(),
          size: file_size,
          sha256_hash,
          url,
          file: shdw_file.file,
      })
    }

    async fn confirm_storage_account_seed(&self, expected_seed: u32, storage_account_key: &Pubkey) -> ShadowDriveResult<u32>{
        let mut num_tries = 0;
        loop {
          let storage_account = self.get_storage_account(storage_account_key).await?;
          if expected_seed == storage_account.init_counter {
            tracing::debug!(
              expected_seed,
              actual_seed = storage_account.init_counter,
              "Chain has up to date info. Moving onto the next batch.");
            return Ok(expected_seed);
          } else if expected_seed < storage_account.init_counter {
            tracing::debug!(
              expected_seed,
              actual_seed = storage_account.init_counter,
              "Chain has higher seed. Fast forwarding to new start.");
              return Ok(storage_account.init_counter);
          } else {
            num_tries += 1;
            if num_tries == 300 {
              // if we've tried for 5 minutes, give up
              return Err(Error::InvalidStorage);
            }

            tracing::debug!(
              expected_seed,
              actual_seed = storage_account.init_counter,
              "Chain does not have up to date info. Waiting 1s to check again.");
            tokio::time::sleep(Duration::from_secs(1)).await;
          }
        }
    }

    async fn send_chunk(&self, storage_account_key: &Pubkey, user_info: Pubkey, new_file_seed: &mut u32, chunk: &[UploadingData]) -> ShadowDriveResult<Vec<ShadowBatchUploadResponse>> {
          let mut files_with_pubkeys: Vec<(Pubkey, &UploadingData)> = Vec::with_capacity(chunk.len());
          for file in chunk {
            files_with_pubkeys.push((derived_addresses::file_account(&storage_account_key, *new_file_seed).0, file));
            *new_file_seed += 1;
          }

          //build txn
          let instructions = files_with_pubkeys.iter().map(|(file_account, file)| {
            let accounts = shdw_drive_accounts::StoreFile {
              storage_config: *STORAGE_CONFIG_PDA,
              storage_account: *storage_account_key,
              user_info,
              owner: self.wallet.pubkey(),
              uploader: UPLOADER,
              token_mint: TOKEN_MINT,
              system_program: system_program::ID,
              file: *file_account
            };
            let args = shdw_drive_instructions::StoreFile {
                filename: file.name.clone(),
                sha256_hash: hex::encode(file.sha256_hash.into_bytes()),
                size: file.size,
            };
            Instruction {
              program_id: PROGRAM_ADDRESS,
              accounts: accounts.to_account_metas(None),
              data: args.data()
            }
           }).collect::<Vec<_>>();

            let mut txn = Transaction::new_with_payer(instructions.as_slice(), Some(&self.wallet.pubkey()));
            txn.try_partial_sign(&[&self.wallet], self.rpc_client.get_latest_blockhash()?)?;
            let txn_encoded = serialize_and_encode(&txn, UiTransactionEncoding::Base64)?;



            let mut form = Form::new();
            for (_, file) in files_with_pubkeys {
              //seek to front of file
              let mut file_data = file.file.try_clone().await.map_err(Error::FileSystemError)?;
              file_data.seek(SeekFrom::Start(0)).await.map_err(Error::FileSystemError)? ;
              form = form.part("file", Part::stream_with_length(file_data, file.size).file_name(file.name.clone()));
            }

            let form = form.part("transaction", Part::text(txn_encoded));

            let response = self.http_client.post(format!("{}/upload-batch",SHDW_DRIVE_ENDPOINT)).multipart(form).send().await?;
            if !response.status().is_success() {
                return Err(Error::ShadowDriveServerError {
                    status: response.status().as_u16(),
                    message: response.json::<Value>().await?,
                });
            }

        let response = response.json::<ShdwDriveBatchServerResponse>().await?;

        let response = chunk.iter().map(|file| {
          ShadowBatchUploadResponse {
            file_name: file.name.clone(),
            status: BatchUploadStatus::Uploaded,
            location: Some(file.url.clone()),
            transaction_signature: Some(response.transaction_signature.clone()),
        }
        }).collect();

        Ok(response)
    }
}
