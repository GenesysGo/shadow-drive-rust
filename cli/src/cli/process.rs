use std::path::PathBuf;
use super::Command;
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use shadow_drive_cli::{FileMetadata, process_shadow_api_response};
use shadow_drive_cli::wait_for_user_confirmation;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::{ShadowDriveClient, StorageAccountVersion};
use shadow_rpc_auth::genesysgo_auth::{authenticate, parse_account_id_from_url};
use shadow_rpc_auth::HttpSenderWithHeaders;
use solana_client::nonblocking;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

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
        &self,
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
                    .create_storage_account(name, size.clone(), StorageAccountVersion::v2())
                    .await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::DeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Delete Storage Account {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_storage_account(storage_account).await;

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
                let response = client.cancel_delete_storage_account(storage_account).await;

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
                let response = client.claim_stake(storage_account).await;

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
                let response = client.reduce_storage(storage_account, size.clone()).await;

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
                let response = client.add_storage(storage_account, size.clone()).await;

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
                    .add_immutable_storage(storage_account, size.clone())
                    .await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::MakeStorageImmutable { storage_account } => {
                let client = shadow_client_factory(signer, rpc_url, auth);
                println!("Make Storage Immutable {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.make_storage_immutable(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetStorageAccount { storage_account } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                println!("Get Storage Account {}", storage_account.to_string());
                let response = client.get_storage_account(storage_account).await;

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
                let response = client.list_objects(storage_account).await;
                let files = process_shadow_api_response(response)?;
                println!("{:#?}", files);
            }
            Command::GetText {
                storage_account,
                filename,
            } => {
                let url = shadow_drive_cli::storage_object_url(storage_account, filename);
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
                let url = shadow_drive_cli::storage_object_url(storage_account, filename);
                println!("Delete file {}", &url);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_file(storage_account, url.clone()).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::EditFile {
                storage_account,
                path,
            } => {
                let client = ShadowDriveClient::new(signer, rpc_url);
                let shadow_file = shadow_file_with_basename(path);
                println!("Edit file {} {}", storage_account.to_string(), path.display());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.edit_file(storage_account, shadow_file).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetObjectData {
                storage_account,
                file,
            } => {
                let url = shadow_drive_cli::storage_object_url(storage_account, file);
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
                for chunk in &files.into_iter().chunks(*batch_size) {
                    let response = client
                        .store_files(
                            &storage_account,
                            chunk
                                .map(|path: &PathBuf| {
                                    shadow_file_with_basename(path)
                                })
                                .collect(),
                        )
                        .await;
                    let resp = process_shadow_api_response(response)?;
                    println!("{:#?}", resp);
                    sleep(Duration::from_millis(150));
                }
            }
        }
        Ok(())
    }
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