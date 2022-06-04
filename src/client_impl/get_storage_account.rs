use anchor_lang::{AccountDeserialize, Discriminator};
use shadow_drive_user_staking::instructions::initialize_account::StorageAccount;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{bs58, pubkey::Pubkey, signer::Signer};

use super::Client;
use crate::{constants::PROGRAM_ADDRESS, error::Error, models::*};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn get_storage_account(&self, key: Pubkey) -> ShadowDriveResult<StorageAccount> {
        let account_info = self.rpc_client.get_account(&key)?;
        StorageAccount::try_deserialize(&mut account_info.data.as_slice())
            .map_err(Error::AnchorError)
    }

    pub async fn get_storage_accounts<'a>(
        &'a self,
        owner: Pubkey,
    ) -> ShadowDriveResult<Vec<StorageAccount>> {
        let account_type_filter = RpcFilterType::Memcmp(Memcmp {
            offset: 0,
            bytes: MemcmpEncodedBytes::Base58(
                bs58::encode(StorageAccount::discriminator()).into_string(),
            ),
            encoding: None,
        });

        let owner_filter = RpcFilterType::Memcmp(Memcmp {
            offset: 39,
            bytes: MemcmpEncodedBytes::Bytes(owner.to_bytes().to_vec()),
            encoding: None,
        });

        let get_accounts_config = RpcProgramAccountsConfig {
            filters: Some(vec![account_type_filter, owner_filter]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                ..RpcAccountInfoConfig::default()
            },
            ..RpcProgramAccountsConfig::default()
        };

        let accounts = self
            .rpc_client
            .get_program_accounts_with_config(&PROGRAM_ADDRESS, get_accounts_config)?;

        let accounts = accounts
            .into_iter()
            .map(|(_, account)| {
                StorageAccount::try_deserialize(&mut account.data.as_slice())
                    .map_err(Error::AnchorError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }
}
