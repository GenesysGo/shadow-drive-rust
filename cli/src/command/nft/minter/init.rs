use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use futures::StreamExt;
use indicatif::ProgressBar;
use inquire::validator::Validation;
use inquire::{Confirm, Text};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{As, DisplayFromStr};
use shadow_drive_sdk::constants::PROGRAM_ADDRESS as SDRIVE_PROGRAM_ADDRESS;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::{Pubkey, Signer, StorageConfig};
use shadow_nft_standard::common::collection::Collection;
use shadow_nft_standard::common::Prefix;
use shadowy_super_minter::accounts::Initialize as InitializeMinterAccounts;
use shadowy_super_minter::instruction::Initialize as InitializeMinterInstruction;
use shadowy_super_minter::instructions::initialize::InitializeArgs as InitializeMinterArgs;
use shadowy_super_minter::state::file_type::{
    AccountDeserialize, AnchorDeserialize, InstructionData, Key, ToAccountMetas,
};
use shadowy_super_minter::state::get_space_for_minter;
use shadowy_super_minter::state::uniform_mint::UniformMint;
use solana_sdk::instruction::Instruction;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

use crate::command::nft::utils::{
     swap_sol_for_shdw_tx, 
    validate_json_compliance, SHDW_MINT_PUBKEY,
};
use crate::utils::shadow_client_factory;

#[derive(Deserialize, Serialize, Debug)]
pub struct MinterInitArgs {
    #[serde(with = "As::<DisplayFromStr>")]
    creator_group: Pubkey,

    #[serde(with = "As::<DisplayFromStr>")]
    collection: Pubkey,

    #[serde(with = "As::<DisplayFromStr>")]
    reveal_hash_all_ones_if_none: Pubkey,

    items_available: u32,
    mint_price_lamports: u64,
    start_time_solana_cluster_time: i64,
    end_time_solana_cluster_time: i64,

    #[serde(with = "As::<DisplayFromStr>")]
    sdrive_account: Pubkey,
    name_prefix: String,

    metadata_dir: PathBuf,
}
impl MinterInitArgs {
    fn template() -> String {
        serde_json::to_string_pretty(&MinterInitArgs {
            creator_group: Pubkey::default(),
            collection: Pubkey::default(),
            reveal_hash_all_ones_if_none: Pubkey::default(),
            items_available: 10000,
            mint_price_lamports: 1_000_000_000,
            start_time_solana_cluster_time: i64::MIN,
            end_time_solana_cluster_time: i64::MAX,
            sdrive_account: Pubkey::default(),
            metadata_dir: Path::new("path").join("to").join("metas"),
            name_prefix: "part of name which is common, e.g. Shadowy Super Coders in Shadowy Super Coders #15000".into(),
        })
        .unwrap()
    }
}

pub(super) async fn process(
    signer: &impl Signer,
    client_signer: impl Signer,
    rpc_url: &str,
) -> anyhow::Result<()> {
    // Construct client
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    println!(
        "This command assumes you have an initialized creator group and for_minter collection"
    );
    println!(
        "To initialize a minter, fill out the following config.json template and provide the path to the config file"
    );
    println!("{:}", MinterInitArgs::template());
    let config_file_path = Text::new("Path to config file:")
        .with_validator(|input: &str| {
            let path = std::path::Path::new(input);
            if path.extension() != Some(OsStr::new("json")) {
                return Ok(Validation::Invalid(
                    "Path does not point to json file".into(),
                ));
            }

            if path.exists() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Path does not exist".into()))
            }
        })
        .prompt()?;

    let Ok(config_file_contents) = std::fs::read_to_string(config_file_path) else {
        return Err(anyhow::Error::msg("Failed to read config json file"))
    };
    let Ok(
        MinterInitArgs { creator_group, collection, reveal_hash_all_ones_if_none, items_available, mint_price_lamports, start_time_solana_cluster_time, end_time_solana_cluster_time, sdrive_account, name_prefix, metadata_dir }
    ) 
    = serde_json::from_str(&config_file_contents) else {
        return Err(anyhow::Error::msg("Failed to deserialize json. Do you have all fields filled in and is it formatted properly?"))
    };
    // TODO, support other prefixes
    let prefix = Prefix::new_sdrive(sdrive_account);

    // Get on-chain collection name
    let collection_name = {
        let Ok(collection_data) = client.get_account_data(&collection).await else {
            return Err(anyhow::Error::msg(format!("No collection account found at {collection}")))
        };

        let mut collection_data_cursor = collection_data.as_slice();
        let Ok(onchain_collection) = Collection::try_deserialize(&mut collection_data_cursor) else {
            return Err(anyhow::Error::msg(format!("Failed to deserialize onchain Collection account")))
        };
        onchain_collection.name
    };

    // Get minter account address
    let Ok(shadowy_super_minter) = Pubkey::create_with_seed(
        &signer.pubkey(),
        &collection.key().to_string()[0..32],
        &shadowy_super_minter::ID,
    ) else {
        return Err(anyhow::Error::msg("Failed to derive minter address"))
    };

    // Prompt user for metadata and image storage method
    let mut files = vec![];
    validate_metadata_dir(&metadata_dir, items_available, &mut files)?;

    let Ok(size_of_all_files) = files.iter().map(|file| file.metadata().map(|meta| meta.len())).fold_ok(0, std::ops::Add::add) else {
        return Err(anyhow::Error::msg("failed to get size of files"))
    };

    match prefix {
        Prefix::ShadowDrive { account } => {
            let sdrive_client = shadow_client_factory(client_signer, rpc_url, None);

            // Try to get sdrive account
            if let Ok(sdrive_account) = sdrive_client.get_storage_account(&account).await {
                // Abort if it's flagged to be deleted
                if sdrive_account.to_be_deleted() {
                    return Err(anyhow::Error::msg(
                        "This storage account is marked for deletion",
                    ));
                }

                // Check if user is owner
                if !sdrive_account.is_owner(signer.pubkey()) {
                    return Err(anyhow::Error::msg("You do not own this storage account"));
                }

                // Check if files exist
                let existing_files = sdrive_client
                    .list_objects(&account)
                    .await
                    .map_err(|_| anyhow::Error::msg("Failed to get files in storage account"))?;
                let all_files_exist = (0..items_available)
                    .all(|i| existing_files.contains(&format!("{i}.json")));

                if !all_files_exist {
                    // Check if there is enough storage
                    if sdrive_account.storage() < size_of_all_files {
                        // If there is not enough space, ask the user if they wish to expand the storage account
                        let user_confirms_expansion = Confirm::new("There is not enough storage in this account. Would you like to expand the storage (This will cost some SHDW)?").prompt()?;
                        if user_confirms_expansion {
                            // Get user SHDW balance from associated token
                            let user_shdw_token_key: Pubkey =
                                spl_associated_token_account::get_associated_token_address(
                                    &signer.pubkey(),
                                    &SHDW_MINT_PUBKEY,
                                );
                            let user_ui_token_amount = client
                                .get_token_account_balance(&user_shdw_token_key)
                                .await?;
                            let Ok(user_shades) = user_ui_token_amount.amount.parse::<u64>() else {
                                        return Err(anyhow::Error::msg("Failed to parse token balance"))
                                    };

                            // Get storage cost
                            let storage_cost_shades_per_gib = {
                                // Fetch and deserialize config account data
                                let config_pubkey = Pubkey::find_program_address(
                                    &["storage-config".as_bytes()],
                                    &SDRIVE_PROGRAM_ADDRESS,
                                )
                                .0;
                                let config_account_data =
                                    client.get_account_data(&config_pubkey).await?;
                                let Ok(config_account) = StorageConfig::deserialize(
                                            &mut config_account_data.as_slice(),
                                        ) else {
                                            return Err(anyhow::Error::msg("Failed to deserialize storage config"))
                                        };
                                config_account.shades_per_gib
                            };

                            // Required shades
                            let requried_storage = size_of_all_files - sdrive_account.storage();
                            let required_shades =
                                safe_amount(requried_storage, storage_cost_shades_per_gib);

                            // Ask for swap if under amount
                            if required_shades < user_shades {
                                let required_ui =
                                    ((required_shades as f64) - (user_shades as f64)) / 1e9;
                                let user_confirms_swap = Confirm::new(&format!("Insufficient SHDW. Authorize jup.ag swap for {required_ui} SHDW?")).prompt()?;

                                if user_confirms_swap {
                                    // Show balance before swap
                                    let user_sol_balance =
                                        client.get_balance(&signer.pubkey()).await? as f64 / 1e9;
                                    let user_shdw_balance_ui = (user_shades as f64) / 1e9;
                                    println!("Current Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW");

                                    // Get swap tx, sign and send.
                                    let mut tx =
                                        swap_sol_for_shdw_tx(required_shades, signer.pubkey())
                                            .await?;
                                    // TODO: show user quote and confirm
                                    tx.signatures[0] = signer.sign_message(&tx.message.serialize());
                                    client.send_and_confirm_transaction(&tx).await?;

                                    // Show balance after swap
                                    let user_sol_balance =
                                        client.get_balance(&signer.pubkey()).await? as f64 / 1e9;
                                    let Ok(user_shades) = user_ui_token_amount.amount.parse::<u64>() else {
                                                    return Err(anyhow::Error::msg("Failed to parse token balance"))
                                                };
                                    let user_shdw_balance_ui = (user_shades as f64) / 1e9;
                                    println!("New Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW");
                                }
                            }

                            if let Err(e) = sdrive_client
                                .add_storage(
                                    &account,
                                    (size_of_all_files - sdrive_account.storage()).into(),
                                )
                                .await
                            {
                                return Err(anyhow::Error::msg(format!(
                                    "Failed to expand storage account\n{e:#?}"
                                )));
                            }
                        } else {
                            return Err(anyhow::Error::msg("Not enough storage in account"));
                        }
                    }

                    // Confirm with user that we will be uploading files
                    let user_confirms_upload = Confirm::new("Upload files to account?").prompt()?;
                    if user_confirms_upload {
                        // Upload all files
                        let pb = ProgressBar::new(files.len() as u64);
                        let futs = files
                            .chunks(5)
                            .map(|files| async {
                                let shdw_files: Vec<ShadowFile> = files
                                    .into_iter()
                                    .map(|file| {
                                        ShadowFile::file(
                                            file.file_name()
                                                .unwrap()
                                                .to_string_lossy()
                                                .into_owned(),
                                            file,
                                        )
                                    })
                                    .collect();
                                let chunk_len = shdw_files.len() as u64;
                                if let Err(e) =
                                    sdrive_client.store_files(&account, shdw_files).await
                                {
                                    return Err(anyhow::Error::msg(format!(
                                        "Failed to upload files\n{e:#?}"
                                    )));
                                }
                                pb.inc(chunk_len);
                                Ok(())
                            })
                            .collect_vec();

                        let results = futures::stream::iter(futs)
                            .buffer_unordered(50)
                            .collect::<Vec<_>>()
                            .await;
                        for result in results {
                            if let Err(e) = result {
                                return Err(anyhow::Error::msg(format!("failed upload {e}")));
                            }
                        }
                        pb.finish();
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Storage account {account} not found"));
            }
        }
        _ => unimplemented!("not yet implemented."),
    };

    // TODO extent to nonuniform mints
    let mint_type = UniformMint {
        reveal_hash: reveal_hash_all_ones_if_none.to_bytes(),
        name_prefix,
        prefix_uri: prefix,
    };

    // Construct the instruction to create a minter
    let args = InitializeMinterArgs {
        price: mint_price_lamports,
        items_available,
        start_time: start_time_solana_cluster_time,
        end_time: end_time_solana_cluster_time,
        // In this command, we always deal with an initialized collection
        if_init_collection: None,
        // In this command, we always deal with an initialized group
        // Note this doesn't allocate!
        if_init_group_name: String::new(),
        mint_type,
    };
    let create_minter_ix_data = InitializeMinterInstruction { args };
    let create_minter_accounts = InitializeMinterAccounts {
        creator_group,
        collection,
        payer_creator: signer.pubkey(),
        system_program: system_program::ID,
        shadowy_super_minter,
        shadow_nft_standard_program: shadow_nft_standard::ID,
    }
    .to_account_metas(None);
    let create_minter_ix = Instruction::new_with_bytes(
        shadowy_super_minter::ID,
        &create_minter_ix_data.data(),
        create_minter_accounts,
    );

    // We need to pay for minter space prior to initialization
    let data_len = get_space_for_minter(
        &create_minter_ix_data.args.mint_type,
        create_minter_ix_data.args.items_available,
    );
    let pay_rent_and_create_account_ix = solana_sdk::system_instruction::create_account_with_seed(
        &signer.pubkey(),
        &shadowy_super_minter,
        &signer.pubkey(),
        &collection.to_string()[0..32],
        client
            .get_minimum_balance_for_rent_exemption(data_len)
            .await?,
        data_len as u64,
        &shadowy_super_minter::ID,
    );

    // Build, sign, send, and confirm transaction
    let create_minter_tx = Transaction::new_signed_with_payer(
        &[pay_rent_and_create_account_ix, create_minter_ix],
        Some(&signer.pubkey()),
        &[signer],
        client.get_latest_blockhash().await?,
    );

    match Confirm::new(&format!("Confirm Input (signing with {})", signer.pubkey())).prompt() {
        Ok(true) => {}
        _ => return Err(anyhow::Error::msg("Discarded Request")),
    }

    if let Err(e) = client.send_and_confirm_transaction(&create_minter_tx).await {
        println!("{e:#?}");
        return Err(anyhow::Error::msg(e));
    };

    println!("Initialized Minter for {collection_name}");

    Ok(())
}

fn validate_metadata_dir(
    path_to_dir: &Path,
    items_available: u32,
    files: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    // Counts the number of files with a name (excluding extension)
    let mut counts = vec![0; items_available as usize];

    // Read all filenames
    let Ok(Ok(all_files_in_dir)) = std::fs::read_dir(path_to_dir)
        .map(|read_dir|
            read_dir
                .map(|file| file.map(|f| f.path()))
                .collect::<Result<Vec<PathBuf>, _>>()
        ) else {
            return Err(anyhow::Error::msg("failed to read directory entries"))
    };

    for i in 0..items_available as usize {
        // Expected filename
        let expected_filename = Path::new(&format!("{i}")).with_extension("json");

        // Read file contents as json
        let Ok(Ok(file_content_as_json)) = std::fs::read_to_string(path_to_dir.join(&expected_filename)).as_deref().map(Value::from_str) else {
            return Err(
                anyhow::Error::msg(format!("failed to read {}", expected_filename.display()))
            )
        };

        // Check if the json is compliant with the standard
        if !validate_json_compliance(&file_content_as_json) {
            return Err(anyhow::Error::msg(format!(
                "{} is not compliant with the standard",
                expected_filename.display()
            )));
        }

        // Check for companion files
        counts[i] += all_files_in_dir
            .iter()
            .filter(|f| f.file_stem() == Some(OsStr::new(&format!("{i}"))))
            .count();
    }

    // Ensure every json file has a companion media file
    for (i, count) in counts.into_iter().enumerate() {
        if count == 1 {
            println!("Warning: File {i}.json does not have a companion media file, e.g. {i}.png")
        }
    }

    *files = all_files_in_dir;

    Ok(())
}

fn safe_amount(additional_storage: u64, rate_per_gib: u64) -> u64 {
    ((additional_storage as u128) * (rate_per_gib as u128) / (BYTES_PER_GIB)) as u64
}
const BYTES_PER_GIB: u128 = 1 << 30;
