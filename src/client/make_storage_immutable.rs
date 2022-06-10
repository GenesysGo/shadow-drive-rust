use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::sysvar::rent;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgramID;

use super::ShadowDriveClient;
use crate::constants::EMISSIONS;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses,
    models::*,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
{
    pub async fn make_storage_immutable(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet_pubkey = self.wallet.pubkey();

        let selected_storage_acct = self.get_storage_account(storage_account_key).await?;
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let emeissions_ata = get_associated_token_address(&EMISSIONS, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(&storage_account_key);

        let accounts = shdw_drive_accounts::MakeAccountImmutable {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            owner: selected_storage_acct.owner_1,
            owner_ata,
            stake_account,
            emissions_wallet: emeissions_ata,
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
