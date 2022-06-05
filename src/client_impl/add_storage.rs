use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use byte_unit::Byte;
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::RpcError,
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgramID;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses,
    error::Error,
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn add_storage<'a>(
        &'a self,
        storage_account_key: Pubkey,
        size: Byte,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let size_as_bytes: u64 = size
            .get_bytes()
            .try_into()
            .map_err(|_| Error::InvalidStorage)?;

        let wallet_pubkey = self.wallet.pubkey();
        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);

        let user_info_acct = self.rpc_client.get_account(&user_info);
        match user_info_acct {
            Ok(_) => {
                // the user_info_acct exists. don't need to verify anything about it as
                // the txn will fail if self.wallet is not the owner of the storage_account
            }
            Err(ClientError {
                kind: ClientErrorKind::RpcError(RpcError::ForUser(_)),
                ..
            }) => {
                // this is what rpc_client.get_account() returns if the account doesn't exist
                // If userInfo hasn't been initialized, error out
                return Err(Error::UserInfoNotCreated);
            }
            Err(err) => {
                //a different rpc error occurred
                return Err(Error::from(err));
            }
        }

        let selected_storage_acct = self.get_storage_account(storage_account_key).await?;
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(&storage_account_key);

        let accounts = shdw_drive_accounts::IncreaseStorage {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: storage_account_key,
            owner: selected_storage_acct.owner_1,
            owner_ata,
            stake_account,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
            token_program: TokenProgramID,
        };
        let args = shdw_drive_instructions::IncreaseStorage {
            additional_storage: size_as_bytes,
        };

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
