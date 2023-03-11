use std::{io::Write, path::PathBuf};

use anchor_lang::{
    prelude::{CpiContext, Program, Pubkey},
    system_program::System,
    ToAccountInfo,
};
use itertools::multizip;
use rkyv::{Archive, CheckBytes, Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod inscribe;

pub use chain_drive::{self, program::ChainDrive, ClockworkInstructionData};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone, CheckBytes)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(rkyv::CheckBytes, Debug))]
#[repr(align(8))]
pub struct Rune {
    pub name: String,
    pub len: u16,
    pub hash: [u8; 32],
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(rkyv::CheckBytes, Debug))]
#[repr(align(8))]
pub struct Runes {
    pub storage_account: [u8; 32],
    pub runes: Vec<Rune>,
}

impl Runes {
    pub fn new(
        storage_account: [u8; 32],
        filenames: Vec<String>,
        filedata: &[Vec<u8>],
        sizes: Vec<usize>,
    ) -> Runes {
        let hashes: Vec<[u8; 32]> = filedata.into_iter().map(sha256_hash).collect();

        // Create runes
        let mut runes = Vec::with_capacity(filenames.len());
        for (filename, size, hash) in multizip((filenames, sizes, hashes)) {
            runes.push(Rune {
                name: filename,
                len: size as u16,
                hash,
            })
        }
        Runes {
            storage_account,
            runes,
        }
    }

    pub fn save(self, mut target: PathBuf) -> Result<(), RunesError> {
        // Serialize
        let bytes = rkyv::to_bytes::<_, 256>(&self).unwrap();

        // Save to file
        target.set_extension("runes");
        let mut file =
            std::fs::File::create(target).map_err(|_| RunesError::FailedToCreateRunesFile)?;
        file.write_all(&bytes)
            .map_err(|_| RunesError::FailedToSaveRunes)?;

        Ok(())
    }
}

impl ArchivedRunes {
    pub fn get_rune(&self, name: &str) -> Option<&ArchivedRune> {
        self.runes.iter().find(|rune| rune.name == name)
    }

    #[allow(unused_must_use)] // cpi failure will make solana runtime panic
    pub fn summon<'info>(
        &self,
        name: &str,
        summoner: impl ToAccountInfo<'info>,
        payer: impl ToAccountInfo<'info>,
        metadata: impl ToAccountInfo<'info>,
        system_program: &Program<'info, System>,
        portal_program: &Program<'info, ChainDrive>,
        signer_seeds: Option<&[&[&[u8]]]>,
        callback: Option<ClockworkInstructionData>,
        extra_lamports: u64,
        unique_thread: u64,
    ) {
        self.get_rune(name).map(|rune| {
            let summoner_info = summoner.to_account_info();
            let cpi_ctx = {
                if let Some(seeds) = signer_seeds {
                    CpiContext::<chain_drive::cpi::accounts::Summon>::new_with_signer(
                        portal_program.to_account_info(),
                        chain_drive::cpi::accounts::Summon {
                            summoner: summoner_info,
                            payer: payer.to_account_info(),
                            metadata: metadata.to_account_info(),
                            system_program: system_program.to_account_info(),
                        },
                        seeds,
                    )
                } else {
                    CpiContext::<chain_drive::cpi::accounts::Summon>::new(
                        portal_program.to_account_info(),
                        chain_drive::cpi::accounts::Summon {
                            summoner: summoner_info,
                            payer: payer.to_account_info(),
                            metadata: metadata.to_account_info(),
                            system_program: system_program.to_account_info(),
                        },
                    )
                }
            };

            let callback_len = callback
                .as_ref()
                .map(|cb| 8 + 34 * cb.accounts.len() + cb.data.len() + 32)
                .unwrap_or(0);
            chain_drive::cpi::summon(
                cpi_ctx,
                Pubkey::new_from_array(self.storage_account),
                rune.name.to_string(),
                rune.len as usize + callback_len,
                rune.hash,
                extra_lamports,
                unique_thread,
                callback,
            );
        });
    }
}

#[derive(Debug)]
pub enum RunesError {
    FailedToCreateRunesFile,
    FailedToSaveRunes,
}

fn sha256_hash(data: &Vec<u8>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .try_into()
        .expect("sha256 is always 32 bytes")
}

#[test]
fn test_rune() {
    let rune = Rune {
        name: "test.txt".to_string(),
        len: 42,
        hash: (0..32).collect::<Vec<u8>>().try_into().unwrap(),
    };

    let bytes = rkyv::to_bytes::<_, 256>(&rune).unwrap();
    println!("{bytes:?}");

    // Or you can use the unsafe API for maximum performance
    let archived = unsafe { rkyv::archived_root::<Rune>(&bytes[..]) };
    assert_eq!(archived, &rune);
}

#[test]
fn test_zero_runes() {
    let runes = Runes {
        storage_account: [0; 32],
        runes: vec![],
    };
    let bytes = rkyv::to_bytes::<_, 256>(&runes).unwrap();
    let archived = unsafe { rkyv::archived_root::<Runes>(&bytes[..]) };
    assert_eq!(archived, &runes);
}

#[test]
fn test_one_runes() {
    let rune = Rune {
        name: "test.txt".to_string(),
        len: 42,
        hash: (0..32).collect::<Vec<u8>>().try_into().unwrap(),
    };

    let runes = Runes {
        storage_account: [0; 32],
        runes: vec![rune],
    };
    let bytes = rkyv::to_bytes::<_, 256>(&runes).unwrap();
    let archived = unsafe { rkyv::archived_root::<Runes>(&bytes[..]) };
    assert_eq!(archived, &runes);
}

#[test]
fn test_two_runes() {
    let rune = Rune {
        name: "test.txt".to_string(),
        len: 42,
        hash: (0..32).collect::<Vec<u8>>().try_into().unwrap(),
    };

    let runes = Runes {
        storage_account: [0; 32],
        runes: vec![rune.clone(), rune],
    };
    let bytes = rkyv::to_bytes::<_, 256>(&runes).unwrap();
    let archived = unsafe { rkyv::archived_root::<Runes>(&bytes[..]) };
    assert_eq!(archived, &runes);
}
