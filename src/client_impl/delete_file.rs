use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use serde_json::json;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::RequestDeleteFile;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use std::borrow::Cow;
use std::str::FromStr;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT},
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn delete_file<'a, 'b>(
        &'a self,
        storage_account_key: &'b Pubkey,
        url: String,
    ) -> ShadowDriveResult<ShdwDriveResponse<'_>> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(&storage_account_key).await?;

        let body = serde_json::to_string(&json!({ "location": url })).unwrap();

        let response = self
            .http_client
            .post(format!("{}/get-object-data", SHDW_DRIVE_ENDPOINT))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let response = response.json::<FileDataResponse>().await?;

        let file_key = Pubkey::from_str(&response.file_data.file_account_pubkey)?;

        let accounts = shdw_drive_accounts::RequestDeleteFile {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            file: file_key,
            owner: selected_account.owner_1,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };

        let args = RequestDeleteFile {};

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
            txid: Cow::from(txn_result.to_string()),
        })
    }
}
