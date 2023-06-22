use crate::utils::{
    get_text, last_modified, parse_filesize, process_shadow_api_response, pubkey_arg,
    shadow_client_factory, shadow_file_with_basename, storage_object_url,
    wait_for_user_confirmation, FileMetadata, FILE_UPLOAD_BATCH_SIZE,
};
use byte_unit::Byte;
use clap::Parser;
use futures::StreamExt;
use shadow_drive_sdk::{Pubkey, ShadowDriveClient, StorageAccountVersion};
use shadow_rpc_auth::genesysgo_auth::{authenticate, parse_account_id_from_url};
use solana_sdk::signature::Signer;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub enum DriveCommand {
    ShadowRpcAuth,
    /// Create an account on which to store data.
    /// Storage accounts can be globally, irreversibly marked immutable
    /// for a one-time fee.
    /// Otherwise, files can be added or deleted from them, and space
    /// rented indefinitely.
    CreateStorageAccount {
        /// Unique identifier for your storage account
        name: String,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Queues a storage account for deletion. While the request is
    /// still enqueued and not yet carried out, a cancellation
    /// can be made (see cancel-delete-storage-account subcommand).
    DeleteStorageAccount {
        /// The account to delete
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Cancels the deletion of a storage account enqueued for deletion.
    CancelDeleteStorageAccount {
        /// The account for which to cancel deletion.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Redeem tokens afforded to a storage account after reducing storage capacity.
    ClaimStake {
        /// The account whose stake to claim.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Increase the capacity of a storage account.
    AddStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Increase the immutable storage capacity of a storage account.
    AddImmutableStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Reduce the capacity of a storage account.
    ReduceStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Make a storage account immutable. This is irreversible.
    MakeStorageImmutable {
        /// Storage account to be marked immutable
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Fetch the metadata pertaining to a storage account.
    GetStorageAccount {
        /// Account whose metadata will be fetched.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Fetch a list of storage accounts owned by a particular pubkey.
    /// If no owner is provided, the configured signer is used.
    GetStorageAccounts {
        /// Searches for storage accounts owned by this owner.
        #[clap(parse(try_from_str = pubkey_arg))]
        owner: Option<Pubkey>,
    },
    /// List all the files in a storage account.
    ListFiles {
        /// Storage account whose files to list.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Get a file, assume it's text, and print it.
    GetText {
        /// Storage account where the file is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to fetch
        filename: String,
    },
    /// Get basic file object data from a storage account file.
    GetObjectData {
        /// Storage account where the file is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to examine.
        file: String,
    },
    /// Delete a file from a storage account.
    DeleteFile {
        /// Storage account where the file to delete is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to delete.
        filename: String,
    },
    /// Has to be the same name as a previously uploaded file
    EditFile {
        /// Storage account where the file to edit is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Path to the new version of the file. Must be the same
        /// name as the file you are editing.
        path: PathBuf,
    },
    /// Upload one or more files to a storage account.
    StoreFiles {
        /// Batch size for file uploads, default 100, only relevant for large
        /// numbers of uploads
        #[clap(long, default_value_t=FILE_UPLOAD_BATCH_SIZE)]
        batch_size: usize,
        /// The storage account on which to upload the files
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// A list of one or more filepaths, each of which is to be uploaded.
        #[clap(min_values = 1)]
        files: Vec<PathBuf>,
    },
}

impl DriveCommand {
    pub async fn process<T: Signer>(
        &self,
        signer: &T,
        client_signer: T,
        rpc_url: &str,
        skip_confirm: bool,
        auth: Option<String>,
    ) -> anyhow::Result<()> {
        let signer_pubkey = signer.pubkey();
        println!("Signing with {:?}", signer_pubkey);
        println!("Sending RPC requests to {}", rpc_url);
        match self {
            DriveCommand::ShadowRpcAuth => {
                let account_id = parse_account_id_from_url(rpc_url.to_string())?;
                let resp = authenticate(signer as &dyn Signer, &account_id).await?;
                println!("{:#?}", resp);
            }
            DriveCommand::CreateStorageAccount { name, size } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!("Create Storage Account {}: {}", name, size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .create_storage_account(name, size.clone(), StorageAccountVersion::v2())
                    .await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::DeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!("Delete Storage Account {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_storage_account(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::CancelDeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!(
                    "Cancellation of Delete Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.cancel_delete_storage_account(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::ClaimStake { storage_account } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!(
                    "Claim Stake on Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.claim_stake(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::ReduceStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!(
                    "Reduce Storage Capacity {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.reduce_storage(storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::AddStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!("Increase Storage {}: {}", storage_account.to_string(), size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.add_storage(storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::AddImmutableStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!(
                    "Increase Immutable Storage {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .add_immutable_storage(storage_account, size.clone())
                    .await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::MakeStorageImmutable { storage_account } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth);
                println!("Make Storage Immutable {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.make_storage_immutable(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::GetStorageAccount { storage_account } => {
                let client = ShadowDriveClient::new(client_signer, rpc_url);
                println!("Get Storage Account {}", storage_account.to_string());
                let response = client.get_storage_account(storage_account).await;

                let act = process_shadow_api_response(response)?;
                println!("{:#?}", act);
            }
            DriveCommand::GetStorageAccounts { owner } => {
                let client = shadow_client_factory(client_signer, rpc_url, auth.clone());
                let owner = owner.as_ref().unwrap_or(&signer_pubkey);
                println!("Get Storage Accounts Owned By {}", owner.to_string());
                let response = client.get_storage_accounts(owner).await;
                let accounts = process_shadow_api_response(response)?;
                println!("{:#?}", accounts);
            }
            DriveCommand::ListFiles { storage_account } => {
                let client = ShadowDriveClient::new(client_signer, rpc_url);
                println!(
                    "List Files for Storage Account {}",
                    storage_account.to_string()
                );
                let response = client.list_objects(storage_account).await;
                let files = process_shadow_api_response(response)?;
                println!("{:#?}", files);
            }
            DriveCommand::GetText {
                storage_account,
                filename,
            } => {
                let url = storage_object_url(storage_account, filename);
                let resp = get_text(&url).await?;
                let last_modified = last_modified(resp.headers())?;
                println!("Get Text at {}", &url);
                println!("Last Modified: {}", last_modified);
                println!("");
                println!("{}", resp.text().await?);
            }
            DriveCommand::DeleteFile {
                storage_account,
                filename,
            } => {
                let client = ShadowDriveClient::new(client_signer, rpc_url);
                let url = storage_object_url(storage_account, filename);
                println!("Delete file {}", &url);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_file(storage_account, url.clone()).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::EditFile {
                storage_account,
                path,
            } => {
                let client = ShadowDriveClient::new(client_signer, rpc_url);
                let shadow_file = shadow_file_with_basename(path);
                println!(
                    "Edit file {} {}",
                    storage_account.to_string(),
                    path.display()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.edit_file(storage_account, shadow_file).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            DriveCommand::GetObjectData {
                storage_account,
                file,
            } => {
                let url = storage_object_url(storage_account, file);
                println!("Get object data {} {}", storage_account.to_string(), file);
                let http_client = reqwest::Client::new();
                let response = http_client.head(url).send().await?;
                let data = FileMetadata::from_headers(response.headers())?;
                println!("{:#?}", data);
            }
            DriveCommand::StoreFiles {
                batch_size,
                storage_account,
                files,
            } => {
                let client = ShadowDriveClient::new(client_signer, rpc_url);
                println!("Store Files {} {:#?}", storage_account.to_string(), files);
                println!(
                    "WARNING: This CLI does not add any encryption on its own. \
                The files in their current state become public as soon as they're uploaded."
                );
                wait_for_user_confirmation(skip_confirm)?;
                let mut responses = Vec::new();
                for chunk in files.chunks(*batch_size) {
                    let response = async {
                        let resp = client
                            .store_files(
                                &storage_account,
                                chunk
                                    .into_iter()
                                    .map(|path: &PathBuf| shadow_file_with_basename(path))
                                    .collect(),
                            )
                            .await;
                        let resp = process_shadow_api_response(resp).unwrap();
                        println!("{:#?}", resp);
                    };
                    responses.push(response);
                }
                futures::stream::iter(responses)
                    .buffer_unordered(100)
                    .collect::<Vec<_>>()
                    .await;
            }
        }
        Ok(())
    }
}
