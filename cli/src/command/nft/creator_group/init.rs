use std::{cell::RefCell, fmt::Display, rc::Rc, str::FromStr};

use inquire::{validator::Validation, Confirm, Select, Text};
use itertools::Itertools;
use shadow_drive_sdk::{Pubkey, Signer};
use shadow_nft_standard::accounts::CreateGroup as CreateGroupAccounts;
use shadow_nft_standard::common::get_creator_group_pda;
use shadow_nft_standard::instruction::CreateGroup as CreateGroupInstruction;
use shadow_nft_standard::instructions::create_group::CreateGroupArgs;
use shadowy_super_minter::state::file_type::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, system_program, transaction::Transaction};
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(PartialEq, Debug, Clone, Copy, EnumString, EnumIter, IntoStaticStr)]
pub enum MemberOptions {
    SingleMember,
    Multisig,
}

pub(crate) async fn process(
    signer: &impl Signer,
    rpc_url: &str,
) -> anyhow::Result<(Pubkey, Vec<Pubkey>)> {
    // Gather information from user about the command
    let options: Vec<MemberOptions> = MemberOptions::iter().collect_vec();

    // Ask user for either single member or multimember
    let Ok(is_single_member) = Select::new(
        "What kind of creator group would you like to create?",
        options,
    ).prompt().map(|option| option == MemberOptions::SingleMember)
     else {
        panic!()
    };

    // Ask for other members if not single_member
    let other_members = Rc::new(RefCell::new(vec![]));
    let other_members_scope = Rc::clone(&other_members);
    let member_label = Rc::new(RefCell::new(1)); // Counter used to label member number
    let member_label_loop = Rc::clone(&member_label);
    let keep_going = Rc::new(RefCell::new(true));
    let keep_going_loop = Rc::clone(&keep_going);
    let signer_pubkey = signer.pubkey();
    if !is_single_member {
        let prompt_text = format!(
            "Add Member {} Pubkey (enter if done):",
            *member_label.borrow() + 1
        );

        // Build the text prompt for collecting peers
        let text_prompt = Text::new(&prompt_text).with_validator(move |input: &str| {
            // Check if done
            if input == "" {
                *keep_going.borrow_mut() = false;
                return Ok(Validation::Valid);
            }

            // Check for valid pubkey
            if let Ok(other_member) = Pubkey::from_str(&*input) {
                // Check for duplicate member
                let is_duplicate_member =
                    other_members.borrow().contains(&other_member) || other_member == signer_pubkey;

                if is_duplicate_member {
                    // Return error if duplicate
                    Ok(Validation::Invalid(
                        inquire::error::InquireError::Custom(
                            "Pubkey already present in group".into(),
                        )
                        .into(),
                    ))
                } else {
                    // Add if valid and not duplicate
                    *member_label.borrow_mut() += 1;
                    other_members.borrow_mut().push(other_member);
                    Ok(Validation::Valid)
                }
            } else {
                Ok(Validation::Invalid("Invalid Pubkey".into()))
            }
        });

        // Gather members until max TODO: replace with crate constant
        while *member_label_loop.borrow() <= 8 && *keep_going_loop.borrow() {
            // Update prompt
            let mut text_prompt_updated = text_prompt.clone();
            let text_prompt_updated_message = format!(
                "Add Member {} Pubkey (enter if done):",
                *member_label_loop.borrow() + 1
            );
            text_prompt_updated.message = &text_prompt_updated_message;

            // Prompt for other member, panic if prompt fails
            if let Err(e) = text_prompt_updated.prompt() {
                return Err(anyhow::Error::msg(e));
            };
        }
        drop(text_prompt.validators);
    }

    // Collect all members and get creator_group
    let (creator_group, all_creators_sorted): (Pubkey, Vec<Pubkey>) = {
        let mut all_creators_sorted = other_members_scope.borrow().clone();
        all_creators_sorted.push(signer.pubkey());
        all_creators_sorted.sort();
        (
            get_creator_group_pda(&all_creators_sorted).expect("length is validated"),
            all_creators_sorted,
        )
    };

    // TODO: add name here when we change contract

    // Confirm input with user
    match Confirm::new(&format!("Confirm Input (signing with {})", signer.pubkey())).prompt() {
        Ok(true) => {}
        _ => return Err(anyhow::Error::msg("Discarded Request")),
    }

    // Construct the instruction to create a creator group
    let args = CreateGroupArgs {
        // TODO: change when multisig is true by default
        multisig: true,
    };
    let create_group_ix_data = CreateGroupInstruction { args };
    let create_group_accounts = CreateGroupAccounts {
        creator_group,
        creator: signer.pubkey(),
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

    println!("Sending create group tx. May take a while to confirm.");
    if let Err(e) = client.send_and_confirm_transaction(&create_group_tx).await {
        return Err(anyhow::Error::msg(e));
    };
    println!("Initialized {creator_group}");

    Ok((creator_group, all_creators_sorted))
}

impl Display for MemberOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}
