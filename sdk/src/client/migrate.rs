use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};

use super::ShadowDriveClient;

use crate::{constants::PROGRAM_ADDRESS, derived_addresses, models::*};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Migrates a v1 [`StorageAccount`](crate::models::StorageAccount) to v2.
    /// This requires two separate transactions to reuse the original pubkey. To minimize chance of failure, it is recommended to call this method with a [commitment level][cl] of [`Finalized`](solana_sdk::commitment_config::CommitmentLevel::Finalized)
    ///
    /// [cl]: https://docs.solana.com/developing/clients/jsonrpc-api#configuring-state-commitment
    ///
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) to be migrated.
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
    ///let migrate_response = shdw_drive_client
    ///     .migrate(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn migrate(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<(ShdwDriveResponse, ShdwDriveResponse)> {
        let step_1_response = self.migrate_step_1(storage_account_key).await?;
        let step_2_response = self.migrate_step_2(storage_account_key).await?;
        Ok((step_1_response, step_2_response))
    }

    /// First transaction step that migrates a v1 [`StorageAccount`](crate::models::StorageAccount) to v2.
    /// Consists of copying the existing account's data into an intermediate account, and deleting the v1 storage account
    pub async fn migrate_step_1(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet_pubkey = self.wallet.pubkey();
        let (migration, _) = derived_addresses::migration_helper(storage_account_key);

        let accounts = shdw_drive_accounts::MigrateStep1 {
            storage_account: *storage_account_key,
            migration,
            owner: wallet_pubkey,
            system_program: system_program::ID,
        };

        let args = shdw_drive_instructions::MigrateStep1 {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        txn.try_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;
        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn).await?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
    /// Second transaction step that migrates a v1 [`StorageAccount`](crate::models::StorageAccount) to v2.
    /// Consists of recreating the storage account using the original pubkey, and deleting the intermediate account
    pub async fn migrate_step_2(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet_pubkey = self.wallet.pubkey();
        let (migration, _) = derived_addresses::migration_helper(storage_account_key);

        let accounts = shdw_drive_accounts::MigrateStep2 {
            storage_account: *storage_account_key,
            migration,
            owner: wallet_pubkey,
            system_program: system_program::ID,
        };

        let args = shdw_drive_instructions::MigrateStep2 {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        txn.try_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;
        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn).await?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
}
