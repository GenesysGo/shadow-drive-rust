use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses::*,
    models::{
        storage_acct::{StorageAccount, StorageAccountV2, StorageAcct},
        *,
    },
};
use spl_token::ID as TokenProgramID;

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Claims any available stake as a result of the `reduce_storage` command.
    /// After reducing storage amount, users must wait until the end of the epoch to successfully claim their stake.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that you want to claim excess stake from.
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
    /// let claim_stake = shdw_drive_client
    ///     .claim_stake(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn claim_stake(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let selected_account = self.get_storage_account(storage_account_key).await?;

        let txn = match selected_account {
            StorageAcct::V1(storage_account) => {
                self.claim_stake_v1(storage_account_key, storage_account)
                    .await?
            }
            StorageAcct::V2(storage_account) => {
                self.claim_stake_v2(storage_account_key, storage_account)
                    .await?
            }
        };

        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn).await?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }

    async fn claim_stake_v1(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccount,
    ) -> ShadowDriveResult<Transaction> {
        let wallet_pubkey = self.wallet.pubkey();
        let unstake_account = unstake_account(storage_account_key).0;
        let unstake_info_account = unstake_info(storage_account_key).0;
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::ClaimStakeV1 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            unstake_info: unstake_info_account,
            unstake_account,
            owner: storage_account.owner_1,
            owner_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };

        let args = shdw_drive_instructions::ClaimStake {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let txn = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet_pubkey),
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        );

        Ok(txn)
    }

    async fn claim_stake_v2(
        &self,
        storage_account_key: &Pubkey,
        storage_account: StorageAccountV2,
    ) -> ShadowDriveResult<Transaction> {
        let wallet_pubkey = self.wallet.pubkey();
        let unstake_account = unstake_account(storage_account_key).0;
        let unstake_info_account = unstake_info(storage_account_key).0;
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::ClaimStakeV2 {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            unstake_info: unstake_info_account,
            unstake_account,
            owner: storage_account.owner_1,
            owner_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };

        let args = shdw_drive_instructions::ClaimStake2 {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let txn = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet_pubkey),
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        );

        Ok(txn)
    }
}
