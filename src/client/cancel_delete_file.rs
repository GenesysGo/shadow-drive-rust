use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::UnmarkDeleteFile;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use std::str::FromStr;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses::stake_account,
    error::Error,
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn cancel_delete_file(
        &self,
        storage_account_key: &Pubkey,
        url: String,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(storage_account_key).await?;
        let stake_account = stake_account(&storage_account_key).0;

        let response = self.get_object_data(&url).await?;

        let file_key = Pubkey::from_str(&response.file_data.file_account_pubkey)?;
        let file_owner = Pubkey::from_str(&response.file_data.owner_account_pubkey)?;
        if file_owner != wallet_pubkey {
            return Err(Error::NotFileOwner);
        }

        let accounts = shdw_drive_accounts::UnmarkDeleteFile {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            file: file_key,
            stake_account,
            owner: selected_account.owner_1,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };

        let args = UnmarkDeleteFile {};

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
