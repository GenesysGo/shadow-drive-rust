use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shadow_drive_sdk::Pubkey;

pub const STATE_FILE_NAME: &'static str = ".shdwclistate";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CliState {
    /// The collection currently being created or managed
    pub collection: Option<Pubkey>,

    /// The path to the keypair currently being used
    pub keypair: Option<PathBuf>,

    /// The current shadow-drive storage account being used
    pub storage_account: Option<Pubkey>,
}

#[test]
fn test_get_home_dir() {
    assert!(
        dirs::home_dir().is_some(),
        "failed to retrieve home directory"
    );
}

#[test]
fn test_clistate_round_trip() {
    use std::io::{Read, Write};
    const LOCAL_TEST_STATE_FILE_NAME: &'static str = ".shadownftroundtriptest";
    std::panic::set_hook(Box::new(|_| {
        // Clean up test
        drop(std::fs::remove_file(LOCAL_TEST_STATE_FILE_NAME));
    }));

    // Define some state
    let state = CliState {
        collection: Some(Pubkey::new_unique()),
        keypair: Some("right_here_bro.json".into()),
        storage_account: Some(Pubkey::new_unique()),
    };

    // Retriefve home directoy
    let Some(home_dir) = dirs::home_dir() else {
        panic!("failed to retrieve home directory");
    };
    println!("{}", home_dir.display());

    // Open state file
    let Ok(mut file) = std::fs::File::create(home_dir.join(LOCAL_TEST_STATE_FILE_NAME)) else {
            panic!("failed to create a test cli state file")
        };

    // Serialize and save state
    let Ok(ser_state) = serde_json::to_string_pretty(&state) else {
        panic!("failed to serialize cli state")
    };
    assert!(
        file.write(ser_state.as_ref()).is_ok(),
        "failed to write cli state"
    );
    drop(file);

    // Read state
    let Ok(mut file) = std::fs::File::open(home_dir.join(LOCAL_TEST_STATE_FILE_NAME)) else {
        panic!("failed to open the newly created test cli state file")
    };
    let mut state_json = String::new();
    assert!(
        file.read_to_string(&mut state_json).is_ok(),
        "failed to read cli state"
    );

    // Deserialize and validate state
    let Ok(deser_state) = serde_json::from_str::<CliState>(&state_json) else {
        panic!("failed to deserialize state");
    };
    assert_eq!(
        &state, &deser_state,
        "deserialized state does not match serialize state"
    );

    // Clean up test
    drop(std::fs::remove_file(LOCAL_TEST_STATE_FILE_NAME));
}
