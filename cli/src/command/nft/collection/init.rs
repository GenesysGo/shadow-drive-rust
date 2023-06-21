use shadow_drive_sdk::{Pubkey, Signer};

use shadow_nft_standard::accounts::CreateCollection as CreateCollectionAccounts;
use shadow_nft_standard::instruction::CreateCollection as CreateCollectionInstruction;
use shadow_nft_standard::instructions::create_collection::CreateCollectionArgs;
use shadowy_super_minter::state::file_type::{InstructionData, ToAccountMetas};
use solana_sdk::instruction::Instruction;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

pub(super) async fn process(signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
    // TODO: get accounts
    let collection = Pubkey::new_unique();
    let creator_group = Pubkey::new_unique();

    // TODO: get args
    let name = "Shadowy".into();
    let symbol = "ssm".into();
    let royalty_50bps = vec![];
    let for_minter = true;

    // Construct the instruction to create a minter
    let args = CreateCollectionArgs {
        name,
        symbol,
        royalty_50bps,
        for_minter,
    };
    let create_group_ix_data = CreateCollectionInstruction { args };
    let create_group_accounts = CreateCollectionAccounts {
        creator_group,
        collection,
        payer_creator: signer.pubkey(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    let create_group_ix = Instruction::new_with_bytes(
        shadow_nft_standard::ID,
        &create_group_ix_data.data(),
        create_group_accounts,
    );

    // Construct client, get latest blockhash, sign and send transaction
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    // Build, sign, send, and confirm transaction
    let create_group_tx = Transaction::new_signed_with_payer(
        &[create_group_ix],
        Some(&signer.pubkey()),
        &[signer],
        client.get_latest_blockhash().await?,
    );

    if let Err(e) = client.send_and_confirm_transaction(&create_group_tx).await {
        return Err(anyhow::Error::msg(e));
    };

    Ok(())
}
