use anchor_lang::{AccountDeserialize, Discriminator};
use shadow_drive_user_staking::instructions::initialize_account::StorageAccount;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{bs58, pubkey::Pubkey, signer::Signer};

use super::ShadowDriveClient;
use crate::{
    constants::PROGRAM_ADDRESS,
    error::Error,
    models::{storage_acct::StorageAcct, *},
};

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
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
        let account_info = self.rpc_client.get_account(&key).await?;
        StorageAcct::deserialize(&mut account_info.data.as_slice()).map_err(Error::AnchorError)
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
    pub fn tmp() {}

    // pub async fn get_storage_accounts(
    //     &self,
    //     owner: &Pubkey,
    // ) -> ShadowDriveResult<Vec<StorageAccount>> {
    //     let account_type_filter = RpcFilterType::Memcmp(Memcmp {
    //         offset: 0,
    //         bytes: MemcmpEncodedBytes::Base58(
    //             bs58::encode(StorageAccount::discriminator()).into_string(),
    //         ),
    //         encoding: None,
    //     });

    //     let owner_filter = RpcFilterType::Memcmp(Memcmp {
    //         offset: 39,
    //         bytes: MemcmpEncodedBytes::Bytes(owner.to_bytes().to_vec()),
    //         encoding: None,
    //     });

    //     let get_accounts_config = RpcProgramAccountsConfig {
    //         filters: Some(vec![account_type_filter, owner_filter]),
    //         account_config: RpcAccountInfoConfig {
    //             encoding: Some(UiAccountEncoding::Base64),
    //             ..RpcAccountInfoConfig::default()
    //         },
    //         ..RpcProgramAccountsConfig::default()
    //     };

    //     let accounts = self
    //         .rpc_client
    //         .get_program_accounts_with_config(&PROGRAM_ADDRESS, get_accounts_config)?;

    //     let accounts = accounts
    //         .into_iter()
    //         .map(|(_, account)| {
    //             StorageAccount::try_deserialize(&mut account.data.as_slice())
    //                 .map_err(Error::AnchorError)
    //         })
    //         .collect::<Result<Vec<_>, _>>()?;

    //     Ok(accounts)
    // }
}
