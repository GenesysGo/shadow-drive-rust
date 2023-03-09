use std::{
    fs::{self, DirEntry},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use anyhow::anyhow;
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use runes::Runes;
use shadow_drive_cli::{process_shadow_api_response, wait_for_user_confirmation, FileMetadata};
use shadow_drive_sdk::{models::ShadowFile, Pubkey, ShadowDriveClient, StorageAccountVersion};
use shadow_rpc_auth::{
    genesysgo_auth::{authenticate, parse_account_id_from_url},
    HttpSenderWithHeaders,
};
use solana_client::{nonblocking, rpc_client::RpcClient};
use solana_sdk::signature::Signer;

use super::Command;

/// We either create an authenticated client with default auth headers,
/// or else we simply use the [RpcClient] provided by the normal
/// [ShadowDriveClient] constructor.
pub fn shadow_client_factory<T: Signer>(
    signer: T,
    url: &str,
    auth: Option<String>,
) -> ShadowDriveClient<T> {
    if let Some(auth) = auth {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str("Authorization").unwrap(),
            HeaderValue::from_str(&format!("Bearer {}", auth)).unwrap(),
        );
        let rpc_client = nonblocking::rpc_client::RpcClient::new_sender(
            HttpSenderWithHeaders::new(url, Some(headers.clone())),
            Default::default(),
        );
        let client = RpcClient::new_sender(
            HttpSenderWithHeaders::new(url, Some(headers)),
            Default::default(),
        );
        let balance = client.get_balance(&signer.pubkey());
        match balance {
            Ok(balance) => {
                println!("{}: {} lamports", signer.pubkey().to_string(), balance);
            }
            Err(e) => {
                println!("Failed to fetch balance: {:?}", e);
            }
        }
        ShadowDriveClient::new_with_rpc(signer, rpc_client)
    } else {
        ShadowDriveClient::new(signer, url)
    }
}

impl Command {
    pub async fn process<T: Signer>(
        self,
        signer: T,
        rpc_url: &str,
        skip_confirm: bool,
        auth: Option<String>,
    ) -> anyhow::Result<()> {
        let signer_pubkey = signer.pubkey();
        println!("Signing with {:?}", signer_pubkey);
        println!("Sending RPC requests to {}", rpc_url);
        match self {
            Command::ShadowRpcAuth => {
                let account_id = parse_account_id_from_url(rpc_url.to_string())?;
                let resp = authenticate(&signer, &account_id).await?;
                println!("{:#?}", resp);
            }
            Command::CreateStorageAccount { name, size } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Create Storage Account {}: {}", name, size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .create_storage_account(&name, size.clone(), StorageAccountVersion::v2())
                    .await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::DeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Delete Storage Account {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_storage_account(&storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::CancelDeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!(
                    "Cancellation of Delete Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.cancel_delete_storage_account(&storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::ClaimStake { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!(
                    "Claim Stake on Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.claim_stake(&storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::ReduceStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!(
                    "Reduce Storage Capacity {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.reduce_storage(&storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::AddStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Increase Storage {}: {}", storage_account.to_string(), size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.add_storage(&storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::AddImmutableStorage {
                storage_account,
                size,
            } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!(
                    "Increase Immutable Storage {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .add_immutable_storage(&storage_account, size.clone())
                    .await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::MakeStorageImmutable { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Make Storage Immutable {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.make_storage_immutable(&storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetStorageAccount { storage_account } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                println!("Get Storage Account {}", storage_account.to_string());
                let response = client.get_storage_account(&storage_account).await;

                let act = process_shadow_api_response(response)?;
                println!("{:#?}", act);
            }
            Command::GetStorageAccounts { owner } => {
                let client = shadow_client_factory(signer, rpc_url, auth.clone());
                let owner = owner.as_ref().unwrap_or(&signer_pubkey);
                println!("Get Storage Accounts Owned By {}", owner.to_string());
                let response = client.get_storage_accounts(owner).await;
                let accounts = process_shadow_api_response(response)?;
                println!("{:#?}", accounts);
            }
            Command::ListFiles { storage_account } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                println!(
                    "List Files for Storage Account {}",
                    storage_account.to_string()
                );
                let response = client.list_objects(&storage_account).await;
                let files = process_shadow_api_response(response)?;
                println!("{:#?}", files);
            }
            Command::GetText {
                storage_account,
                filename,
            } => {
                let url = shadow_drive_cli::storage_object_url(&storage_account, &filename);
                let resp = shadow_drive_cli::get_text(&url).await?;
                let last_modified = shadow_drive_cli::last_modified(resp.headers())?;
                println!("Get Text at {}", &url);
                println!("Last Modified: {}", last_modified);
                println!("");
                println!("{}", resp.text().await?);
            }
            Command::DeleteFile {
                storage_account,
                filename,
            } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                let url = shadow_drive_cli::storage_object_url(&storage_account, &filename);
                println!("Delete file {}", &url);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_file(&storage_account, url.clone()).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::EditFile {
                storage_account,
                path,
            } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                let shadow_file = shadow_file_with_basename(&path);
                println!(
                    "Edit file {} {}",
                    storage_account.to_string(),
                    path.display()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.edit_file(&storage_account, shadow_file).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetObjectData {
                storage_account,
                file,
            } => {
                let url = shadow_drive_cli::storage_object_url(&storage_account, &file);
                println!("Get object data {} {}", storage_account.to_string(), file);
                let http_client = reqwest::Client::new();
                let response = http_client.head(url).send().await?;
                let data = FileMetadata::from_headers(response.headers())?;
                println!("{:#?}", data);
            }
            Command::StoreFiles {
                batch_size,
                storage_account,
                files,
            } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                println!("Store Files {} {:#?}", storage_account.to_string(), files);
                println!(
                    "WARNING: This CLI does not add any encryption on its own. \
                The files in their current state become public as soon as they're uploaded."
                );
                wait_for_user_confirmation(skip_confirm)?;
                for chunk in &files.iter().chunks(batch_size) {
                    let response = client
                        .store_files(
                            &storage_account,
                            chunk
                                .map(|path: &PathBuf| shadow_file_with_basename(path))
                                .collect(),
                        )
                        .await;
                    let resp = process_shadow_api_response(response)?;
                    println!("{:#?}", resp);
                    sleep(Duration::from_millis(150));
                }
            }
            Command::StoreAndCreateRunes { directory, target } => {

                // Get the paths and sizes of files in the given directory
                // NOTE: this checks that all file sizes are under MAX_FILE_SIZE
                let (paths, filesizes) = get_paths_and_sizes(&directory)?;
                let total_bytes = filesizes.iter().sum::<usize>() as u64;

                // Check if target exists
                if target.exists() {
                    return Err(anyhow!("{target:?} already exists"));
                }
                if let Some(parent) = target.parent() {
                    if !parent.eq(Path::new("")) {
                        fs::create_dir_all(parent);
                    }
                }

                // Check user has enough SHDW
                let client = ShadowDriveClient::new(signer, rpc_url);
                let (storage_price, min_size) = client
                    .get_storage_price_and_min_account_size()
                    .await
                    .map_err(|e| anyhow!("{e:?}"))?;

                let cost: u64 = (total_bytes.max(min_size) as u128)
                    .checked_mul(storage_price as u128)
                    .unwrap()
                    .checked_div(2u128.pow(30) as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();

                let balance: u64 = client
                    .get_shdw_balance()
                    .await
                    .map_err(|e| anyhow!("{e:?}"))?;

                if cost > balance {
                    return Err(anyhow!(
                        "Insufficient funds, cost = {cost:?}, balance = {balance:?}"
                    ));
                }

                // Create storage account with target name
                let target_name = target.file_name().unwrap().to_string_lossy();
                let response = client
                    .create_storage_account(
                        &target_name,
                        total_bytes.max(min_size).into(),
                        StorageAccountVersion::V2,
                    )
                    .await;
                let storage_account: Pubkey =
                    Pubkey::from_str(&process_shadow_api_response(response)?.shdw_bucket.unwrap())
                        .unwrap();
                println!("Created storage account");

                // Load all filedata
                let (filedata, filenames): (Vec<Vec<u8>>, Vec<String>) = paths
                    .iter()
                    .zip(&filesizes)
                    .map(|(path, &size)| {
                        let mut buffer = Vec::with_capacity(size);
                        let mut file = std::fs::File::open(path).expect("file has been checked");
                        file.read_to_end(&mut buffer)
                            .expect("file has been checked");
                        (
                            buffer,
                            path.file_name().unwrap().to_str().unwrap().to_string(),
                        )
                    })
                    .unzip();

                // Generate runes
                let runes = Runes::new(
                    storage_account.to_bytes(),
                    filenames.clone(),
                    &filedata,
                    filesizes.clone(),
                );

                // Upload data to account
                let shadow_files: Vec<ShadowFile> = filedata
                    .into_iter()
                    .zip(filenames)
                    .map(|(data, name)| ShadowFile::bytes(name, data))
                    .collect();
                process_shadow_api_response(
                    client.store_files(&storage_account, shadow_files).await,
                )?;
                println!("Uploaded data to Shadow Drive.");

                // Save runes to disk
                runes
                    .save(target)
                    .map_err(|e| anyhow!("failed to save runes {e:?}"))?;
            }
        }
        Ok(())
    }
}

pub const MAX_FILE_SIZE: usize = 1000;
/// Gets the paths and sizes of files in the given directory
fn get_paths_and_sizes(directory: &PathBuf) -> anyhow::Result<(Vec<PathBuf>, Vec<usize>)> {
    let mut filesizes = vec![];
    let mut paths = vec![];
    if let Ok(paths_iter) = fs::read_dir(directory) {
        for path in paths_iter {
            if let Ok(path) = path {
                if let Ok(metadata) = path.metadata() {
                    if (metadata.len() as usize) > MAX_FILE_SIZE {
                        return Err(anyhow!("{path:?} is larger than {MAX_FILE_SIZE} bytes"));
                    }
                    filesizes.push(metadata.len() as usize);
                    paths.push(path.path());
                } else {
                    return Err(anyhow!("could not get metadata for {path:?}"));
                }
            } else {
                return Err(anyhow!("{path:?} not valid or does not exist"));
            }
        }
    } else {
        return Err(anyhow!("{directory:?} not valid or does not exist"));
    }
    Ok((paths, filesizes))
}

// TODO Maybe make this a result type.
/// Factory function for a [ShadowFile], where we just use the path's
/// basename. Panics if `path.file_name()` returns None.
pub fn shadow_file_with_basename(path: &PathBuf) -> ShadowFile {
    let basename = {
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string()
    };
    ShadowFile::file(basename, path.clone())
}
