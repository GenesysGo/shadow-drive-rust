use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::ClaimStake;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses::*,
    models::*,
};
use spl_token::ID as TokenProgramID;

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
{
    /// Claims any available stake as a result of the `reduce_storage` command.
    /// After reducing storage amount, users must wait until the end of the epoch to successfully claim their stake.
    /// * `storage_account_key` - The public key of the [`StorageAccount`] that you want to claim excess stake from.
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
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(storage_account_key).await?;
        let unstake_account = unstake_account(&storage_account_key).0;
        let unstake_info_account = unstake_info(&storage_account_key).0;
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::ClaimStake {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            unstake_info: unstake_info_account,
            unstake_account,
            owner: selected_account.owner_1,
            owner_ata,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };

        let args = ClaimStake {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let txn = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet_pubkey),
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash()?,
        );

        let txn_result = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &txn,
                CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                },
            )?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
}
