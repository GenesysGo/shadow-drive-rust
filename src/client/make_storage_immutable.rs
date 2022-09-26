use anchor_lang::{system_program, InstructionData, ToAccountMetas};
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
    models::{
        storage_acct::{StorageAccount, StorageAccountV2, StorageAcct},
        ShadowDriveResult, StorageResponse,
    },
    serialize_and_encode,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Permanently locks a [`StorageAccount`](crate::models::StorageAccount) and all contained files. After a [`StorageAccount`](crate::models::StorageAccount)
    /// has been locked, a user will no longer be able to delete/edit files, add/reduce storage amount,
    /// or delete the [`StorageAccount`](crate::models::StorageAccount).
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that will be made immutable.
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
    /// let make_immutable_response = shdw_drive_client
    ///     .make_storage_immutable(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn make_storage_immutable(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<StorageResponse> {
        let selected_storage_acct = self.get_storage_account(storage_account_key).await?;

        let txn_encoded = match selected_storage_acct {
            StorageAcct::V1(storage_account) => {
                self.make_storage_immutable_v1(storage_account_key, storage_account)
                    .await?
            }
            StorageAcct::V2(storage_account) => {
                self.make_storage_immutable_v2(storage_account_key, storage_account)
                    .await?
            }
        };

        self.send_shdw_txn("make-immutable", txn_encoded).await
    }

    async fn make_storage_immutable_v1(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccount,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let emissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(storage_account_key);

        let accounts = shdw_drive_accounts::MakeAccountImmutableV1 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            owner: storage_account.owner_1,
            uploader: UPLOADER,
            owner_ata,
            stake_account,
            emissions_wallet: emissions_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
            associated_token_program: spl_associated_token_account::ID,
            rent: rent::ID,
        };

        let args = shdw_drive_instructions::MakeAccountImmutable {};

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

    async fn make_storage_immutable_v2(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccountV2,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let emissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(storage_account_key);

        let accounts = shdw_drive_accounts::MakeAccountImmutableV2 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            owner: storage_account.owner_1,
            uploader: UPLOADER,
            owner_ata,
            stake_account,
            emissions_wallet: emissions_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
            associated_token_program: spl_associated_token_account::ID,
            rent: rent::ID,
        };
        let args = shdw_drive_instructions::MakeAccountImmutable2 {};

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
