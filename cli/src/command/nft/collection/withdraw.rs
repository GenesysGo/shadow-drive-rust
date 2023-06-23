use inquire::Confirm;
use shadow_drive_sdk::{Pubkey, Signer};
use shadow_nft_standard::{
    accounts::Withdraw as WithdrawAccounts,
    common::{collection::Collection, creator_group::CreatorGroup},
    instruction::Withdraw as WithdrawInstruction,
};
use shadowy_super_minter::state::file_type::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    transaction::Transaction,
};

pub(crate) async fn process(
    signer: &impl Signer,
    collection: Pubkey,
    rpc_url: &str,
) -> Result<(), anyhow::Error> {
    let client = RpcClient::new(rpc_url);

    // Get onchain data
    let onchain_collection =
        Collection::try_deserialize(&mut client.get_account_data(&collection)?.as_slice())?;
    let onchain_creator_group = CreatorGroup::try_deserialize(
        &mut client
            .get_account_data(&onchain_collection.creator_group_key)?
            .as_slice(),
    )?;
    let creator_group = onchain_collection.creator_group_key;

    // Build tx
    let mut accounts = WithdrawAccounts {
        payer_creator: signer.pubkey(),
        collection,
        creator_group,
    }
    .to_account_metas(None);
    for creator in onchain_creator_group.creators {
        if creator != signer.pubkey() {
            accounts.push(AccountMeta::new(creator, false))
        }
    }
    let withdraw_tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            shadow_nft_standard::ID,
            WithdrawInstruction {}.data().as_ref(),
            WithdrawAccounts {
                payer_creator: signer.pubkey(),
                collection,
                creator_group,
            }
            .to_account_metas(None),
        )],
        Some(&signer.pubkey()),
        &[signer],
        client.get_latest_blockhash()?,
    );

    // Confirm with user
    match Confirm::new(&format!(
        "Send and confirm transaction (signing with {})?",
        signer.pubkey()
    ))
    .prompt()
    {
        Ok(true) => {}
        _ => return Err(anyhow::Error::msg("Discarded Request")),
    }

    // Sign and send
    if let Err(e) = client.send_and_confirm_transaction(&withdraw_tx) {
        return Err(anyhow::Error::msg(e));
    };

    Ok(())
}
