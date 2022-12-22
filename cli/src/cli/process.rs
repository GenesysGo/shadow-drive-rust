use super::Command;
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use shadow_rpc_auth::genesysgo_auth::{parse_account_id_from_url, sign_in};
use shadow_rpc_auth::HttpSenderWithHeaders;
use shadow_drive_cli::process_shadow_api_response;
use shadow_drive_cli::wait_for_user_confirmation;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::{ShadowDriveClient, StorageAccountVersion};
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
        url: &str,
        skip_confirm: bool,
        auth: Option<String>,
    ) -> anyhow::Result<()> {
        let signer_pubkey = signer.pubkey();
        println!("Signing with {:?}", signer_pubkey);
        println!("Sending RPC requests to {}", url);
        match self {
            Command::ShadowRpcAuth => {
                let account_id = parse_account_id_from_url(url.to_string())?;
                let resp = sign_in(&signer, &account_id).await?;
                println!("{:#?}", resp);
            }
            Command::CreateStorageAccount { name, size } => {
                let client = shadow_client_factory(signer, url, auth);
                println!("Create Storage Account {}: {}", name, size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .create_storage_account(name, size.clone(), StorageAccountVersion::v2())
                    .await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::DeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(signer, url, auth);
                println!("Delete Storage Account {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_storage_account(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::CancelDeleteStorageAccount { storage_account } => {
                let client = shadow_client_factory(signer, url, auth);
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
                let client = shadow_client_factory(signer, url, auth);
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
                let client = shadow_client_factory(signer, url, auth);
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
                let client = shadow_client_factory(signer, url, auth);
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
                let client = shadow_client_factory(signer, url, auth);
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
                let client = shadow_client_factory(signer, url, auth);
                println!("Make Storage Immutable {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.make_storage_immutable(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetStorageAccount { storage_account } => {
                let client = ShadowDriveClient::new(signer, url);
                println!("Get Storage Account {}", storage_account.to_string());
                let response = client.get_storage_account(storage_account).await;

                let act = process_shadow_api_response(response)?;
                println!("{:#?}", act);
            }
            Command::GetStorageAccounts { owner } => {
                let client = shadow_client_factory(signer, url, auth.clone());
                let owner = owner.as_ref().unwrap_or(&signer_pubkey);
                println!("Get Storage Accounts Owned By {}", owner.to_string());
                let response = client.get_storage_accounts(owner).await;
                let accounts = process_shadow_api_response(response)?;
                println!("{:#?}", accounts);
            }
            Command::ListFiles { storage_account } => {
                let client = ShadowDriveClient::new(signer, url);
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
                file,
            } => {
                let location = shadow_drive_cli::drive_url(storage_account, file);
                let resp = shadow_drive_cli::get_text(&location).await?;
                let last_modified = shadow_drive_cli::last_modified(resp.headers())?;
                println!("Get Text at {}", &location);
                println!("Last Modified: {}", last_modified);
                println!("");
                println!("{}", resp.text().await?);
            }
            Command::DeleteFile {
                storage_account,
                file,
            } => {
                let client = ShadowDriveClient::new(signer, url);
                let location = shadow_drive_cli::drive_url(storage_account, file);
                println!("Delete file {}", &location);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_file(storage_account, location.clone()).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::EditFile {
                storage_account,
                file,
            } => {
                let client = ShadowDriveClient::new(signer, url);
                let basename = shadow_drive_cli::acquire_basename(file);
                let shdw_file = ShadowFile::file(basename, file.clone());
                println!("Edit file {} {}", storage_account.to_string(), file);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.edit_file(storage_account, shdw_file).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetObjectData {
                storage_account,
                file,
            } => {
                let client = ShadowDriveClient::new(signer, url);
                let location = shadow_drive_cli::drive_url(storage_account, file);
                println!("Get object data {} {}", storage_account.to_string(), file);
                let response = client.get_object_data(&location).await;
                let data = process_shadow_api_response(response)?;
                println!("{:#?}", data);
            }
            Command::StoreFiles {
                batch_size,
                storage_account,
                files,
            } => {
                let client = ShadowDriveClient::new(signer, url);
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
                                .map(|s| {
                                    let basename = shadow_drive_cli::acquire_basename(s);
                                    ShadowFile::file(basename, s.clone())
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
