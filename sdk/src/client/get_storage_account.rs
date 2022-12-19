use anchor_lang::AccountDeserialize;
use futures::future::join_all;
use serde_json::json;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use super::ShadowDriveClient;
use crate::{
    constants::SHDW_DRIVE_ENDPOINT,
    derived_addresses,
    models::{storage_acct::StorageAcct, *},
};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Returns the [`StorageAccount`](crate::models::StorageAccount) associated with the pubkey provided by a user.
    /// * `key` - The public key of the [`StorageAccount`](crate::models::StorageAccount).
    ///
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
    /// #
    /// let storage_account = shdw_drive_client
    ///     .get_storage_account(&storage_account_key)
    ///     .await
    ///     .expect("failed to get storage account");
    /// ```
    pub async fn get_storage_account(&self, key: &Pubkey) -> ShadowDriveResult<StorageAcct> {
        let response = self
            .http_client
            .post(format!("{}/storage-account-info", SHDW_DRIVE_ENDPOINT))
            .json(&json!({
                "storage_account": key.to_string()
            }))
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Returns all [`StorageAccount`]s associated with the public key provided by a user.
    /// * `owner` - The public key that is the owner of all the returned [`StorageAccount`]s.
    ///
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::ShadowDriveClient;
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
    /// #
    /// let storage_accounts = shdw_drive_client
    ///     .get_storage_accounts(&user_pubkey)
    ///     .await
    ///     .expect("failed to get storage account");
    /// ```
    pub async fn get_storage_accounts(
        &self,
        owner: &Pubkey,
    ) -> ShadowDriveResult<Vec<StorageAcct>> {
        let (user_info_key, _) = derived_addresses::user_info(owner);
        let user_info = self.rpc_client.get_account_data(&user_info_key).await?;
        let user_info = UserInfo::try_deserialize(&mut user_info.as_slice())?;

        let accounts_to_fetch = (0..user_info.account_counter)
            .map(|account_seed| derived_addresses::storage_account(owner, account_seed).0);

        let accounts = accounts_to_fetch.map(|storage_account_key| async move {
            self.get_storage_account(&storage_account_key).await
        });

        let (accounts, errors): (
            Vec<ShadowDriveResult<StorageAcct>>,
            Vec<ShadowDriveResult<StorageAcct>>,
        ) = join_all(accounts)
            .await
            .into_iter()
            .partition(Result::is_ok);

        tracing::debug!(?errors, "encountered errors fetching storage_accounts");

        //unwrap is safe due do the abve partition
        Ok(accounts.into_iter().map(Result::unwrap).collect())
    }
}
