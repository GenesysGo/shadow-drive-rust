use inquire::Confirm;
use shadow_drive_sdk::{Keypair, Pubkey, Signer};
use shadow_nft_common::get_payer_pda;
use shadow_nft_standard::common::collection::Collection;
use shadow_nft_standard::common::creator_group::CreatorGroup;
use shadow_nft_standard::common::{token_2022, Metadata};
use shadowy_super_minter::accounts::Mint as MintAccounts;
use shadowy_super_minter::instruction::Mint as MintInstruction;
use shadowy_super_minter::state::file_type::{AccountDeserialize, InstructionData};
use shadowy_super_minter::state::file_type::{Id, ToAccountMetas};
use shadowy_super_minter::state::ShadowySuperMinter;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::{system_program, sysvar};

pub(super) async fn process(
    signer: &impl Signer,
    shadowy_super_minter: Pubkey,
    rpc_url: &str,
) -> anyhow::Result<()> {
    // Get onchain data
    let client = RpcClient::new(rpc_url);
    let onchain_shadowy_super_minter = ShadowySuperMinter::try_deserialize(
        &mut client.get_account_data(&shadowy_super_minter)?.as_slice(),
    )?;
    let collection = onchain_shadowy_super_minter.collection;
    let onchain_collection =
        Collection::try_deserialize(&mut client.get_account_data(&collection)?.as_slice())?;
    let creator_group = onchain_shadowy_super_minter.creator_group;
    let onchain_creator_group =
        CreatorGroup::try_deserialize(&mut client.get_account_data(&creator_group)?.as_slice())?;

    // Build mint tx
    let mint_keypair = Keypair::new(); // never used after this
    let mint_tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            shadowy_super_minter::ID,
            MintInstruction {}.data().as_ref(),
            MintAccounts {
                shadowy_super_minter,
                minter: signer.pubkey(),
                minter_ata: spl_associated_token_account::get_associated_token_address(
                    &signer.pubkey(),
                    &mint_keypair.pubkey(),
                ),
                payer_pda: get_payer_pda(&mint_keypair.pubkey()),
                mint: mint_keypair.pubkey(),
                collection,
                metadata: Metadata::derive_pda(&mint_keypair.pubkey()),
                creator_group,
                shadow_nft_standard: shadow_nft_standard::ID,
                token_program: token_2022::Token2022::id(),
                associated_token_program: spl_associated_token_account::ID,
                system_program: system_program::ID,
                recent_slothashes: sysvar::slot_hashes::ID,
            }
            .to_account_metas(None),
        )],
        Some(&signer.pubkey()),
        &[signer as &dyn Signer, &mint_keypair as &dyn Signer],
        client.get_latest_blockhash()?,
    );

    // Confirm with user
    println!("Minting an NFT from:)");
    println!("    minter        {shadowy_super_minter}");
    #[rustfmt::skip]
    println!("    collection    {} ({collection})", &onchain_collection.name);
    #[rustfmt::skip]
    println!("    creator_group {} ({})", &onchain_creator_group.name, creator_group);
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
    match client.send_and_confirm_transaction(&mint_tx) {
        Ok(sig) => {
            println!("Successful: https://explorer.solana.com/tx/{sig}")
        }
        Err(e) => return Err(anyhow::Error::msg(e)),
    };

    println!("minted");

    Ok(())
}
