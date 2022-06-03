use std::borrow::Cow;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::RequestDeleteAccount;
use solana_sdk::{
    instruction::Instruction, signer::Signer, pubkey::Pubkey, transaction::Transaction,
};

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn request_delete_storage_account<'a>(
        &'a self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse<'_>> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(&storage_account_key).await?;

        let accounts = shdw_drive_accounts::RequestDeleteAccount {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            owner: selected_account.owner_1,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };

        let args = RequestDeleteAccount { };

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
