use lazy_static::lazy_static;
use solana_sdk::{pubkey, pubkey::Pubkey};

/// Address of the Mainnet Shadow Drive Program.
pub static PROGRAM_ADDRESS: Pubkey = pubkey!("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ");
/// Address of the Mainnet Shadow Token Mint.
pub static TOKEN_MINT: Pubkey = pubkey!("SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y");
/// Address of the upload authority for the Mainnet Shadow Drive Program.
pub static UPLOADER: Pubkey = pubkey!("972oJTFyjmVNsWM4GHEGPWUomAiJf2qrVotLtwnKmWem");
/// Address that handles token emissions for the Mainnet Shadow Drive Program.
pub static EMISSIONS: Pubkey = pubkey!("SHDWRWMZ6kmRG9CvKFSD7kVcnUqXMtd3SaMrLvWscbj");

lazy_static! {
    /// Program Derived Address that holds storage config parameters and admin pubkeys for the Mainnet Shadow Drive Program.
    pub static ref STORAGE_CONFIG_PDA: Pubkey =
        Pubkey::find_program_address(&[b"storage-config"], &PROGRAM_ADDRESS).0;
}
/// Endpoint that is used for file uploads and fetching object data.
pub const SHDW_DRIVE_ENDPOINT: &str = "https://shadow-storage.genesysgo.net";
pub const SHDW_DRIVE_OBJECT_PREFIX: &str = "https://shdw-drive.genesysgo.net";

pub const FILE_SIZE_LIMIT: u64 = 1_073_741_824; //1GB
