use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use chrono::Local;
use inquire::validator::Validation;
use inquire::{max_length, Confirm, Select, Text};
use itertools::Itertools;
use serde::Serialize;
use serde_json::Value;
use shadow_drive_sdk::constants::PROGRAM_ADDRESS as SDRIVE_PROGRAM_ADDRESS;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::{Pubkey, Signer, StorageConfig};
use shadow_nft_standard::common::collection::Collection;
use shadow_nft_standard::common::creator_group::CreatorGroup;
use shadow_nft_standard::common::Prefix;
use shadow_nft_standard::instructions::create_collection::CreateCollectionArgs;
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
use strum::IntoEnumIterator;

use crate::command::nft::utils::{
    pubkey_validator, swap_sol_for_shdw_tx, validate_and_convert_to_half_percent,
    validate_json_compliance, SHDW_MINT_PUBKEY,
};
use crate::utils::shadow_client_factory;

#[derive(Serialize)]
pub struct MinterInitArgs {
    init_creator_group: bool,
    creator_group: String,
}

/// This impl kind of works but is not robust. kinda finnicky
pub(super) async fn process(
    signer: &impl Signer,
    client_signer: impl Signer,
    rpc_url: &str,
) -> anyhow::Result<()> {
    // Construct client
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    // Ask user if they have a creator group already.
    let Ok(has_creator_group) =
        inquire::Confirm::new("Are you part of a creator group (single member or multisig)?")
            .prompt() else {
                return Err(anyhow::Error::msg("Cancelled Request"))
            };

    // Get or make creator group
    let (creator_group, all_members_sorted): (Pubkey, Vec<Pubkey>) = {
        if has_creator_group {
            let pubkey_str = Text::new("Creator Group Pubkey:")
                .with_validator(pubkey_validator)
                .prompt()
                .map_err(|_| anyhow::Error::msg("Cancelled Request"))?;
            let creator_group = Pubkey::from_str(&pubkey_str).unwrap();
            (
                creator_group,
                get_creators_from_group(creator_group, rpc_url).await?,
            )
        } else {
            super::super::creator_group::init::process(signer, rpc_url).await?
        }
    };

    // Ask user if they have an existing collection
    let Ok(has_existing_collection) =
        inquire::Confirm::new("Have you already initialized a collection for this minter?")
            .prompt() else {
                return Err(anyhow::Error::msg("Cancelled Request"))
            };

    // Get the collection pubkey
    let mut collection_name = String::new();
    let (collection, if_init_collection): (Pubkey, Option<CreateCollectionArgs>) = {
        if has_existing_collection {
            // Get collection if already initialized
            let pubkey_str = Text::new("Collection Pubkey:")
                .with_validator(pubkey_validator)
                .prompt()
                .map_err(|_| anyhow::Error::msg("Cancelled Request"))?;
            let collection = Pubkey::from_str(&pubkey_str).unwrap();

            // Validate this collection belongs to this creator
            validate_existing_collection(collection, creator_group, &mut collection_name, rpc_url)
                .await?;

            (collection, None)
        } else {
            // Build init collection args
            let royalty_text_prompt = Text::new("").with_validator(&|input: &str| {
                if validate_and_convert_to_half_percent(&*input).is_ok() {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("asdf".into()))
                }
            });
            let args = CreateCollectionArgs {
                for_minter: true,
                name: Text::new("What do you want to name your collection?").prompt()?,
                symbol: Text::new("What symbol (e.g. SOL) do you want to use?")
                    .with_validator(max_length!(8, "Symbol has a max length of 8"))
                    .prompt()?,
                royalty_50bps: all_members_sorted
                    .iter()
                    .map(|creator| {
                        let prompt_text = format!("Royalty (in multiples of 0.5%) for {creator}:");
                        let mut tp = royalty_text_prompt.clone();
                        tp.message = &prompt_text;
                        tp.prompt()
                            .map(|s| validate_and_convert_to_half_percent(&s))
                            .map(Result::unwrap)
                    })
                    .collect::<Result<_, _>>()?,
            };
            let collection = Collection::get_pda(creator_group, &args.name);
            collection_name = args.name.clone();

            match Confirm::new(&format!(
                "Confirm royalties {:.1?}",
                &args
                    .royalty_50bps
                    .iter()
                    .map(|b| *b as f32 / 2.0)
                    .collect_vec()
            ))
            .prompt()
            {
                Ok(true) => {}
                _ => return Err(anyhow::Error::msg("Discarded Request")),
            }

            // Check collection does not exist
            validate_inexistent_collection(collection, rpc_url).await?;

            (collection, Some(args))
        }
    };

    // Get minter account address
    let Ok(shadowy_super_minter) = Pubkey::create_with_seed(
        &signer.pubkey(),
        &collection.key().to_string()[0..32],
        &shadowy_super_minter::ID,
    ) else {
        return Err(anyhow::Error::msg("Failed to derive minter address"))
    };

    // Get minter parameters
    // First get price in SOL from the user
    let price: u64 = Text::new("Mint Price (in SOL):")
        .with_placeholder("1.0")
        .with_validator(&|input: &str| {
            // Get price as a f64
            let Ok(price_floating) = str::parse::<f64>(input) else {
                return Ok(Validation::Invalid("Not a valid number".into()))
            };

            // Convert to lamports
            if let Err(e) = convert_f64_to_u64(price_floating) {
                return Ok(Validation::Invalid(e.into()));
            }

            Ok(Validation::Valid)
        })
        .prompt()
        .map(|s| str::parse::<f64>(&s))
        .map(Result::unwrap)
        .map(convert_f64_to_u64)
        .map(Result::unwrap)?;

    // Then get number of items to be minted from the user
    let items_available: u32 = Text::new("Number of Assets:")
        .with_placeholder("1000")
        .with_validator(Box::new(|input: &str| {
            if input.parse::<u32>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Invalid Integer Input".into()))
            }
        }))
        .prompt()
        .map(|s| str::parse(&s))
        .map(Result::unwrap)?;

    // Get start time from user
    let start_time_text = Text::new("Mint Start Time (Solana Cluster DateTime, blank for ASAP):")
        .with_validator(&|input: &str| {
            if chrono::DateTime::<Local>::from_str(input).is_ok() || input.trim() == "" {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Invalid Time".into()))
            }
        })
        .prompt()?;
    let start_time: i64 = if start_time_text.trim() == "" {
        // Get latest block time if empty string
        client.get_block_time(client.get_slot().await?).await?
    } else {
        // Parse unix timestamp (already validated)
        chrono::DateTime::<Local>::from_str(&start_time_text)
            .unwrap()
            .timestamp()
    };

    // Get end time from user
    let validator = move |input: &str| {
        if let Ok(time) = chrono::DateTime::<Local>::from_str(input) {
            if time.timestamp() > start_time {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "End Time comes before Start Time".into(),
                ))
            }
        } else if input.trim() == "" {
            // Perpetual mint
            Ok(Validation::Valid)
        } else {
            // Nonsense input
            Ok(Validation::Invalid("Invalid Time".into()))
        }
    };
    let end_time_text = Text::new("Mint End Time (Solana Cluster DateTime, blank for perpetual):")
        .with_validator(validator)
        .prompt()?;
    let end_time: i64 = if end_time_text.trim() == "" {
        // Perpetual if empty
        i64::MAX
    } else {
        // Parse unix timestamp (already validated)
        chrono::DateTime::<Local>::from_str(&start_time_text)
            .unwrap()
            .timestamp()
    };

    // Prompt user for metadata and image storage method
    let files = Rc::new(RefCell::new(vec![]));
    let files_in_closure = Rc::clone(&files);
    let metadata_directory =
        Text::new("Provide the path to the directory containing metadata and images")
            .with_validator(move |input: &str| {
                // Ensure it is a valid path to a directory
                let path = Path::new(input);
                // Note: is_dir checks for existence
                if path.is_dir() {
                    validate_metadata_dir(path, items_available, &mut files_in_closure.borrow_mut())
                } else {
                    Ok(Validation::Invalid(
                        "Path does not exist or is not a directory".into(),
                    ))
                }
            })
            .prompt()?;
    let Ok(size_of_all_files) = files.borrow().iter().map(|file| file.metadata().map(|meta| meta.len())).fold_ok(0, std::ops::Add::add) else {
        return Err(anyhow::Error::msg("failed to get size of files"))
    };

    let prefix_options: Vec<Prefix> = Prefix::iter().collect();
    let mut prefix: Prefix = Select::new(
        "What Storage Option will you be using (requires deterministic prefix + filename)",
        prefix_options,
    )
    .prompt()?;
    match &mut prefix {
        Prefix::ShadowDrive { ref mut account } => {
            // Prompt user and ask if they have a ShadowDrive account
            let has_storage_account =
                Confirm::new("Do you have an existing Shadow Drive account you want to use?")
                    .prompt()?;
            let sdrive_client = shadow_client_factory(client_signer, rpc_url, None);

            if has_storage_account {
                // Ask user for storage account. Can be name or pubkey
                let storage_account_str =
                    Text::new("What storage account do you want to use (provide name or Pubkey)?")
                        .prompt()?;

                // Check if they provided a valid storage account
                if let Ok(storage_account_pubkey) = Pubkey::from_str(&storage_account_str) {
                    // Try to get sdrive account
                    if let Ok(sdrive_account) = sdrive_client
                        .get_storage_account(&storage_account_pubkey)
                        .await
                    {
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
                            .list_objects(&storage_account_pubkey)
                            .await
                            .map_err(|_| {
                                anyhow::Error::msg("Failed to get files in storage account")
                            })?;
                        let all_files_exist = (0..items_available)
                            .all(|i| existing_files.contains(&format!("{i}.json")))
                            & (existing_files.len() > 2 * items_available as usize);

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
                                    let requried_storage =
                                        size_of_all_files - sdrive_account.storage();
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
                                                client.get_balance(&signer.pubkey()).await? as f64
                                                    / 1e9;
                                            let user_shdw_balance_ui = (user_shades as f64) / 1e9;
                                            println!(
                                                "Current Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW",
                                            );

                                            // Get swap tx, sign and send.
                                            let mut tx = swap_sol_for_shdw_tx(
                                                required_shades,
                                                signer.pubkey(),
                                            )
                                            .await?;
                                            // TODO: show user quote and confirm
                                            tx.signatures[0] =
                                                signer.sign_message(&tx.message.serialize());
                                            client.send_and_confirm_transaction(&tx).await?;

                                            // Show balance after swap
                                            let user_sol_balance =
                                                client.get_balance(&signer.pubkey()).await? as f64
                                                    / 1e9;
                                            let Ok(user_shades) = user_ui_token_amount.amount.parse::<u64>() else {
                                                    return Err(anyhow::Error::msg("Failed to parse token balance"))
                                                };
                                            let user_shdw_balance_ui = (user_shades as f64) / 1e9;
                                            println!(
                                                "New Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW",
                                            );
                                        }
                                    }

                                    if let Err(e) = sdrive_client
                                        .add_storage(
                                            &storage_account_pubkey,
                                            (size_of_all_files - sdrive_account.storage()).into(),
                                        )
                                        .await
                                    {
                                        return Err(anyhow::Error::msg(format!(
                                            "Failed to expand storage account\n{e:#?}"
                                        )));
                                    }
                                } else {
                                    return Err(anyhow::Error::msg(
                                        "Not enough storage in account",
                                    ));
                                }
                            }

                            // Confirm with user that we will be uploading files
                            let user_confirms_upload =
                                Confirm::new("Upload files to account?").prompt()?;
                            if user_confirms_upload {
                                // Upload all files
                                let shdw_files = files
                                    .borrow()
                                    .iter()
                                    .map(|file| {
                                        ShadowFile::file(file.to_string_lossy().into_owned(), file)
                                    })
                                    .collect_vec();
                                if let Err(e) = sdrive_client
                                    .store_files(&storage_account_pubkey, shdw_files)
                                    .await
                                {
                                    return Err(anyhow::Error::msg(format!(
                                        "Failed to upload files\n{e:#?}"
                                    )));
                                };
                            }
                        }

                        // Write to account in prefix
                        *account = storage_account_pubkey;
                    }
                }
            } else {
                // Initialize a storage account
                let user_confirms_init_and_upload = Confirm::new(
                    "Would you like to initialize one (will cost SHDW) and upload files?",
                )
                .prompt()?;
                if user_confirms_init_and_upload {
                    // TODO START REFACTOR
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
                        let config_account_data = client.get_account_data(&config_pubkey).await?;
                        let Ok(config_account) = StorageConfig::deserialize(
                        &mut config_account_data.as_slice(),
                    ) else {
                        return Err(anyhow::Error::msg("Failed to deserialize storage config"))
                    };
                        config_account.shades_per_gib
                    };

                    // Required shades
                    let requried_storage = size_of_all_files;
                    let required_shades =
                        safe_amount(requried_storage, storage_cost_shades_per_gib);

                    // Ask for swap if under amount
                    if required_shades < user_shades {
                        let required_ui = ((required_shades as f64) - (user_shades as f64)) / 1e9;
                        let user_confirms_swap = Confirm::new(&format!(
                            "Insufficient SHDW. Authorize jup.ag swap for {required_ui} SHDW?"
                        ))
                        .prompt()?;

                        if user_confirms_swap {
                            // Show balance before swap
                            let user_sol_balance =
                                client.get_balance(&signer.pubkey()).await? as f64 / 1e9;
                            let user_shdw_balance_ui = (user_shades as f64) / 1e9;
                            println!(
                            "Current Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW",
                        );

                            // Get swap tx, sign and send.
                            let mut tx =
                                swap_sol_for_shdw_tx(required_shades, signer.pubkey()).await?;
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
                            println!(
                                "New Balance {user_sol_balance} SOL, {user_shdw_balance_ui} SHDW",
                            );
                        }
                    }
                    // TODO END REFACTOR

                    // Create storage account
                    let response = match sdrive_client.create_storage_account(&collection_name, size_of_all_files.into(), shadow_drive_sdk::StorageAccountVersion::V2).await {
                        Ok(account) => account,
                        Err(e) => return Err(anyhow::Error::msg(format!("Failed to initialize sdrive account. Make sure you have enough SHDW.\n{e:#?}")))
                    };

                    // Parse pubkey
                    let storage_account_pubkey =
                        Pubkey::from_str(&response.shdw_bucket.expect("transaction succeeded"))
                            .expect("transaction succeeded");

                    // Upload all files
                    let shdw_files = files
                        .borrow()
                        .iter()
                        .map(|file| ShadowFile::file(file.to_string_lossy().into_owned(), file))
                        .collect_vec();
                    if let Err(e) = sdrive_client
                        .store_files(&storage_account_pubkey, shdw_files)
                        .await
                    {
                        return Err(anyhow::Error::msg(format!(
                            "Failed to upload files\n{e:#?}"
                        )));
                    };

                    // Write to account in prefix
                    *account = storage_account_pubkey;
                } else {
                    return Err(anyhow::Error::msg("Discarded User Request"));
                }
            }
        }
        _ => unimplemented!("not yet implemented."),
    };

    let reveal_hash: [u8; 32] = {
        // Ask if mint will have postmint reveal
        let mint_has_postmint_reveal =
            Confirm::new("Will this mint involve a post-mint reveal?").prompt()?;

        if mint_has_postmint_reveal {
            // TODO
            unimplemented!("not yet implemented");
        } else {
            [0; 32]
        }
    };
    match Confirm::new(&format!("Confirm Input (signing with {})", signer.pubkey())).prompt() {
        Ok(true) => {}
        _ => return Err(anyhow::Error::msg("Discarded Request")),
    }

    // TODO extent to nonuniform mints
    let mint_type = UniformMint {
        reveal_hash,
        name_prefix: Text::new(
            "What name prefix (e.g. \"Llama\" in \"Llama #1\" would you like to use for the minted items",
        )
        .prompt()?,
        prefix_uri: Prefix::Arweave,
    };

    // Construct the instruction to create a minter
    let args = InitializeMinterArgs {
        price,
        items_available,
        start_time,
        end_time,
        if_init_collection,
        // In this cli, we always deal with an initialized group
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
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    // Counts the number of files with a name (excluding extension)
    let mut counts = vec![0; items_available as usize];

    // Read all filenames
    let Ok(Ok(all_files_in_dir)) = std::fs::read_dir(path_to_dir)
        .map(|read_dir|
            read_dir
                .map(|file| file.map(|f| f.path()))
                .collect::<Result<Vec<PathBuf>, _>>()
        ) else {
            return Ok(Validation::Invalid("failed to read directory entries".into()))
    };

    for i in 0..items_available as usize {
        // Expected filename
        let expected_filename = Path::new(&format!("{i}")).with_extension("json");

        // Read file contents as json
        let Ok(Ok(file_content_as_json)) = std::fs::read_to_string(path_to_dir.join(&expected_filename)).as_deref().map(Value::from_str) else {
            return Ok(Validation::Invalid(
                format!("failed to read {}", expected_filename.display()).into(),
            ))
        };

        // Check if the json is compliant with the standard
        if !validate_json_compliance(&file_content_as_json) {
            return Ok(Validation::Invalid(
                format!(
                    "{} is not compliant with the standard",
                    expected_filename.display()
                )
                .into(),
            ));
        }

        // Check for companion files
        counts[i] += all_files_in_dir
            .iter()
            .filter(|f| f.file_stem() == Some(OsStr::new(&format!("i"))))
            .count();
    }

    // Ensure every json file has a companion media file
    for (i, count) in counts.into_iter().enumerate() {
        if count >= 2 {
            return Ok(Validation::Invalid(
                format!("File {i}.json does not have a companion media file").into(),
            ));
        }
    }

    *files = all_files_in_dir;

    Ok(Validation::Valid)
}

async fn validate_inexistent_collection(collection: Pubkey, rpc_url: &str) -> anyhow::Result<()> {
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
    match client.get_account_data(&collection).await {
        Ok(_) => Err(anyhow::Error::msg("Collection already exists")),
        Err(_) => Ok(()),
    }
}

/// Validates that a collection exists, belongs to this `creator_group`, and has not previously has a mint.
/// Also writes the onchain collection name to the `name` string.
async fn validate_existing_collection(
    collection: Pubkey,
    creator_group: Pubkey,
    name: &mut String,
    rpc_url: &str,
) -> anyhow::Result<()> {
    // Fetch and deserialize account data
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
    let account_data = client
        .get_account_data(&creator_group)
        .await
        .map_err(|_| anyhow::Error::msg(format!("Creator Group {creator_group} does not exist")))?;
    let onchain_collection = Collection::try_deserialize(&mut account_data.as_slice())?;

    // Check if on-chain creator matches provided creator
    if onchain_collection.creator_group_key != creator_group {
        return Err(anyhow::Error::msg(format!(
            "Collection {collection} does not belong to {creator_group}"
        )));
    }

    // Check if collection is empty (there have already been some mints)
    if onchain_collection.size != 0 {
        return Err(anyhow::Error::msg(format!(
            "Collection {collection} is not empty"
        )));
    }

    *name = onchain_collection.name;

    Ok(())
}

async fn get_creators_from_group(
    creator_group: Pubkey,
    rpc_url: &str,
) -> anyhow::Result<Vec<Pubkey>> {
    // Fetch and deserialize account data
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());
    let account_data = client
        .get_account_data(&creator_group)
        .await
        .map_err(|_| anyhow::Error::msg(format!("Creator Group {creator_group} does not exist")))?;
    let onchain_creator_group = CreatorGroup::try_deserialize(&mut account_data.as_slice())?;

    Ok(onchain_creator_group.creators)
}

fn convert_f64_to_u64(sol_balance: f64) -> Result<u64, &'static str> {
    // Check if sol_balance is negative
    if sol_balance < 0.0 {
        return Err("Input SOL is negative");
    }

    // Check if the value exceeds u64::MAX
    if sol_balance > u64::MAX as f64 {
        return Err("Error: sol_balance exceeds u64::MAX");
    }

    // Perform the conversion and return the result
    Ok(sol_balance as u64)
}

fn safe_amount(additional_storage: u64, rate_per_gib: u64) -> u64 {
    ((additional_storage as u128) * (rate_per_gib as u128) / (BYTES_PER_GIB)) as u64
}
const BYTES_PER_GIB: u128 = 1 << 30;
