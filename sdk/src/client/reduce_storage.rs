use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use byte_unit::Byte;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_sdk::sysvar::rent;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgramID;

use super::ShadowDriveClient;
use crate::{
    constants::{EMISSIONS, PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    derived_addresses,
    error::Error,
    models::{
        storage_acct::{StorageAccount, StorageAccountV2, StorageAcct},
        *,
    },
    serialize_and_encode,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Reduces the amount of total storage available for the given storage account.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) whose storage will be reduced.
    /// * `size` - The amount of storage you want to remove.
    /// E.g if you have an existing [`StorageAccount`](crate::models::StorageAccount) with 3MB of storage
    /// but you want 2MB total, `size` should equal 1MB.
    /// When specifying size, only KB, MB, and GB storage units are currently supported.
    ///
    /// # Example
    ///
    /// ```
    /// # use byte_unit::Byte;
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
    /// # let reduced_bytes = Byte::from_str("1MB").expect("invalid byte string");
    /// #
    /// let reduce_storage_response = shdw_drive_client
    ///     .reduce_storage(&storage_account_key, reduced_bytes)
    ///     .await?;
    /// ```
    pub async fn reduce_storage(
        &self,
        storage_account_key: &Pubkey,
        size: Byte,
    ) -> ShadowDriveResult<StorageResponse> {
        let size_as_bytes: u64 = size
            .get_bytes()
            .try_into()
            .map_err(|_| Error::InvalidStorage)?;

        let selected_storage_acct = self.get_storage_account(storage_account_key).await?;

        let txn_encoded = match selected_storage_acct {
            StorageAcct::V1(storage_account) => {
                self.reduce_storage_v1(storage_account_key, storage_account, size_as_bytes)
                    .await?
            }
            StorageAcct::V2(storage_account) => {
                self.reduce_storage_v2(storage_account_key, storage_account, size_as_bytes)
                    .await?
            }
        };

        self.send_shdw_txn("reduce-storage", txn_encoded).await
    }

    async fn reduce_storage_v1(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccount,
        size_as_bytes: u64,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();
        let (unstake_account, _) = derived_addresses::unstake_account(storage_account_key);
        let (unstake_info, _) = derived_addresses::unstake_info(storage_account_key);

        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(storage_account_key);

        let emeissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::DecreaseStorageV1 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            unstake_info,
            unstake_account,
            owner: storage_account.owner_1,
            owner_ata,
            stake_account,
            uploader: UPLOADER,
            emissions_wallet: emeissions_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
            rent: rent::ID,
        };
        let args = shdw_drive_instructions::DecreaseStorage {
            remove_storage: size_as_bytes,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }

    async fn reduce_storage_v2(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccountV2,
        size_as_bytes: u64,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();
        let (unstake_account, _) = derived_addresses::unstake_account(storage_account_key);
        let (unstake_info, _) = derived_addresses::unstake_info(storage_account_key);

        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(storage_account_key);

        let emeissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::DecreaseStorageV2 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            unstake_info,
            unstake_account,
            owner: storage_account.owner_1,
            owner_ata,
            stake_account,
            uploader: UPLOADER,
            emissions_wallet: emeissions_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
            rent: rent::ID,
        };
        let args = shdw_drive_instructions::DecreaseStorage2 {
            remove_storage: size_as_bytes,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }
}
