use std::str::FromStr;

use inquire::{Confirm, Text};
use shadow_drive_sdk::{Pubkey, Signer};

use shadow_nft_standard::common::collection::Collection;
use shadow_nft_standard::instruction::CreateCollection as CreateCollectionInstruction;
use shadow_nft_standard::instructions::create_collection::CreateCollectionArgs;
use shadow_nft_standard::{
    accounts::CreateCollection as CreateCollectionAccounts, common::creator_group::CreatorGroup,
};
use shadowy_super_minter::state::file_type::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

use crate::command::nft::utils::{pubkey_validator, validate_and_convert_to_half_percent};

pub(super) async fn process(signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
    let client = RpcClient::new(rpc_url);
    let creator_group = Pubkey::from_str(
        &Text::new("For which creator group are you initializing a collection?")
            .with_validator(pubkey_validator)
            .prompt()?,
    )
    .unwrap();
    let Ok(Ok(onchain_creator_group)) = client
        .get_account_data(&creator_group)
        .map(|data| CreatorGroup::try_deserialize(&mut data.as_slice())) else {
        return Err(anyhow::Error::msg("Failed to retrieve on-chain creator group. Check input or create a group!"))
    };

    // Get arguments for the collection
    let name = Text::new("What would you like to name your collection?").prompt()?;
    let symbol =
        Text::new("What symbol (e.g. USDC) would you like to use for the collection?").prompt()?;
    let text_prompt = Text::new("");
    let royalty_50bps = onchain_creator_group
        .creators
        .iter()
        .map(|creator| {
            let prompt_text = format!("Royalty (in multiples of 0.5%) for {creator}:");
            let mut tp = text_prompt.clone();
            tp.message = &prompt_text;
            tp.prompt()
                .map(|s| validate_and_convert_to_half_percent(&*s))
                .map(Result::unwrap)
        })
        .collect::<Result<_, _>>()?;

    let collection = Collection::get_pda(creator_group, &name);

    // Construct the instruction to create a minter
    let for_minter = Confirm::new("Is this collection for a shadowy super minter? (no for 1/1s). You cannot change this later").prompt()?;
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
    println!("");

    Ok(())
}
