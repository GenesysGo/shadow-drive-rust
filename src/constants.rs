use lazy_static::lazy_static;
use solana_sdk::{pubkey, pubkey::Pubkey};

pub static PROGRAM_ADDRESS: Pubkey = pubkey!("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ");
pub static TOKEN_MINT: Pubkey = pubkey!("SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y");
pub static UPLOADER: Pubkey = pubkey!("972oJTFyjmVNsWM4GHEGPWUomAiJf2qrVotLtwnKmWem");
pub static EMISSIONS: Pubkey = pubkey!("SHDWRWMZ6kmRG9CvKFSD7kVcnUqXMtd3SaMrLvWscbj");

lazy_static! {
    pub static ref STORAGE_CONFIG_PDA: Pubkey =
        Pubkey::find_program_address(&[b"storage-config"], &PROGRAM_ADDRESS).0;
}

pub const SHDW_DRIVE_ENDPOINT: &'static str = "https://shadow-storage.genesysgo.net";
