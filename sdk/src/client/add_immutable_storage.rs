use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use byte_unit::Byte;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;

use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgramID;

use super::ShadowDriveClient;
use crate::models::storage_acct::{StorageAccount, StorageAccountV2, StorageAcct};
use crate::serialize_and_encode;
use crate::{
    constants::{EMISSIONS, PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    error::Error,
    models::*,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Adds storage capacity to the specified immutable [`StorageAccount`](crate::models::StorageAccount).
    /// This will fail if the [`StorageAccount`](crate::models::StorageAccount) is not immutable.
    /// * `storage_account_key` - The public key of the immutable [`StorageAccount`](crate::models::StorageAccount).
    /// * `size` - The additional amount of storage you want to add.
    /// E.g if you have an existing [`StorageAccount`](crate::models::StorageAccount) with 1MB of storage
    /// but you need 2MB total, `size` should equal 1MB.
    /// When specifying size, only KB, MB, and GB storage units are currently supported.
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
    /// let add_immutable_storage_response = shdw_drive_client
    ///     .add_immutable_storage(storage_account_key, Byte::from_str("1MB").expect("invalid byte string"))
    ///     .await?;
    /// ```
    pub async fn add_immutable_storage(
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
                if !storage_account.immutable {
                    return Err(Error::StorageAccountIsNotImmutable);
                }
                self.add_immutable_storage_v1(storage_account_key, storage_account, size_as_bytes)
                    .await?
            }
            StorageAcct::V2(storage_account) => {
                if !storage_account.immutable {
                    return Err(Error::StorageAccountIsNotImmutable);
                }
                self.add_immutable_storage_v2(storage_account_key, storage_account, size_as_bytes)
                    .await?
            }
        };

        self.send_shdw_txn("add-storage", txn_encoded).await
    }

    async fn add_immutable_storage_v1(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccount,
        size_as_bytes: u64,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = &self.wallet.pubkey();
        let owner_ata = get_associated_token_address(wallet_pubkey, &TOKEN_MINT);
        let emissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::IncreaseImmutableStorageV1 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            emissions_wallet: emissions_ata,
            owner: storage_account.owner_1,
            owner_ata,
            uploader: UPLOADER,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };
        let args = shdw_drive_instructions::IncreaseImmutableStorage {
            additional_storage: size_as_bytes,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(wallet_pubkey));

        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }

    async fn add_immutable_storage_v2(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccountV2,
        size_as_bytes: u64,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = &self.wallet.pubkey();
        let owner_ata = get_associated_token_address(wallet_pubkey, &TOKEN_MINT);
        let emissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::IncreaseImmutableStorageV2 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            emissions_wallet: emissions_ata,
            owner: storage_account.owner_1,
            owner_ata,
            uploader: UPLOADER,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };

        let args = shdw_drive_instructions::IncreaseImmutableStorage2 {
            additional_storage: size_as_bytes,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(wallet_pubkey));

        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }
}
