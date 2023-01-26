use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::{
    pymodule,
    types::{PyModule, PyString},
    PyResult, Python,
};
use shadow_drive_sdk::constants::SHDW_DRIVE_OBJECT_PREFIX;
use shadow_drive_sdk::models::{ShadowFile, ShadowUploadResponse};
use shadow_drive_sdk::{
    read_keypair_file, Byte, CommitmentConfig, Keypair, Pubkey, RpcClient,
    ShadowDriveClient as ShadowDriveRustClient, Signer,
};
use tokio::runtime::{Builder, Runtime};

/// A Python module implemented in Rust.
#[pymodule]
fn shadow_drive(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add Solana Mainnet-Beta RPC endpoint
    const SOLANA_MAINNET_BETA: &'static str = "https://api.mainnet-beta.solana.com";
    m.add("SOLANA_MAINNET_BETA", SOLANA_MAINNET_BETA)?;

    #[pyclass]
    pub struct ShadowDriveClient {
        rust_client: Arc<ShadowDriveRustClient<Keypair>>,
        runtime: Runtime,
        current_account: Option<Pubkey>,
    }

    #[pymethods]
    impl ShadowDriveClient {
        /// new(keypair, account/)
        /// --
        ///
        /// ShadowDriveClient constructor. By default, this uses confirmed commitment and the Solana Labs
        /// public RPC endpoint. To use a custom commitment level, use the new_with_commitment method. To
        /// use a custom RPC endpoint, use the new_with_rpc method. Or, to specify both, use the method
        /// new_with_commitment_and_rpc.
        #[new]
        fn new(keypair: &str, account: Option<&str>) -> PyResult<ShadowDriveClient> {
            let keypair: Keypair = read_keypair_file(keypair)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read keypair file {e}")))?;

            let rust_client = Arc::new(ShadowDriveRustClient::new(
                keypair,
                SOLANA_MAINNET_BETA.to_string(),
            ));
            let runtime = Builder::new_multi_thread()
                .worker_threads(2)
                .enable_time()
                .enable_io()
                .build()
                .unwrap();
            let current_account: Option<Pubkey> =
                check_current_account(account, &rust_client, &runtime);

            Ok(ShadowDriveClient {
                rust_client,
                runtime,
                current_account,
            })
        }

        /// new_with_commitment(keypair, commitment, /)
        /// --
        ///
        /// ShadowDriveClient constructor. Specify one of 'processed', 'confirmed', or 'finalized'
        /// for the commitment level.
        fn new_with_commitment(
            keypair: Py<PyAny>,
            commitment: &str,
            account: Option<&str>,
        ) -> PyResult<ShadowDriveClient> {
            // Extract commitment
            let commitment_config = extract_commitment(commitment)?;

            let keypair: Keypair = read_keypair_file(keypair.to_string())
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read keypair file {e}")))?;

            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                keypair,
                RpcClient::new_with_commitment(SOLANA_MAINNET_BETA.to_string(), commitment_config),
            ));

            let runtime = Builder::new_multi_thread()
                .worker_threads(2)
                .enable_time()
                .enable_io()
                .build()
                .unwrap();
            let current_account: Option<Pubkey> =
                check_current_account(account, &rust_client, &runtime);

            Ok(ShadowDriveClient {
                rust_client,
                runtime,
                current_account,
            })
        }

        /// new_with_rpc(keypair, rpc, /)
        /// --
        ///
        /// ShadowDriveClient constructor. Specify a custom RPC endpoint. Uses finalized commitment level.
        fn new_with_rpc(
            keypair: Py<PyAny>,
            rpc: &str,
            account: Option<&str>,
        ) -> PyResult<ShadowDriveClient> {
            let keypair: Keypair = read_keypair_file(keypair.to_string())
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read keypair file {e}")))?;

            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                keypair,
                RpcClient::new_with_commitment(rpc.to_string(), CommitmentConfig::finalized()),
            ));

            let runtime = Builder::new_multi_thread()
                .worker_threads(2)
                .enable_time()
                .enable_io()
                .build()
                .unwrap();
            let current_account: Option<Pubkey> =
                check_current_account(account, &rust_client, &runtime);

            Ok(ShadowDriveClient {
                rust_client,
                runtime,
                current_account,
            })
        }

        /// new_with_commitment_and_rpc(keypair, commitment, rpc, /)
        /// --
        ///
        /// ShadowDriveClient constructor. Specify one of 'processed', 'confirmed', or 'finalized'
        /// for the commitment level, and a custom RPC enpdoint.
        fn new_with_commitment_and_rpc(
            keypair: Py<PyAny>,
            commitment: &str,
            rpc: &str,
            account: Option<&str>,
            py: Python,
        ) -> PyResult<ShadowDriveClient> {
            // Extract commitment
            let commitment_config = extract_commitment(commitment)?;

            let keypair: Keypair = read_keypair_file(keypair.to_string())
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to read keypair file {e}")))?;

            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                keypair,
                RpcClient::new_with_commitment(rpc.to_string(), commitment_config),
            ));

            let runtime = Builder::new_multi_thread()
                .worker_threads(2)
                .enable_time()
                .enable_io()
                .build()
                .unwrap();
            let current_account: Option<Pubkey> =
                check_current_account(account, &rust_client, &runtime);

            Ok(ShadowDriveClient {
                rust_client,
                runtime,
                current_account,
            })
        }

        /// create_account(name, size, rpc, /)
        /// --
        ///
        /// Create a Shadow Drive storage account with the specified name and number of bytes.
        fn create_account(
            &mut self,
            name: &str,
            size: u64,
            use_account: Option<bool>,
            py: Python,
        ) -> PyResult<(Py<PyString>, Py<PyString>)> {
            let result: PyResult<(Py<PyString>, Py<PyString>)> = self.runtime.block_on(async {
                self.rust_client
                    .create_storage_account(
                        name,
                        Byte::from(size as u128),
                        shadow_drive_sdk::StorageAccountVersion::V2,
                    )
                    .await
                    .map(|response| {
                        (
                            PyString::new(py, &response.shdw_bucket.unwrap()).into(),
                            PyString::new(py, &response.transaction_signature).into(),
                        )
                    })
                    .map_err(|err| {
                        PyValueError::new_err(format!("failed to create storage account {err:?}"))
                            .into()
                    })
            });
            if let Ok((ref bucket, _)) = result {
                if let Some(true) = use_account {
                    self.current_account = Some(
                        Pubkey::from_str(&bucket.to_string())
                            .expect("sucessful storage account creation"),
                    );
                }
            }
            result
        }

        /// delete_account(name, size, rpc, /)
        /// --
        ///
        /// Delete a Shadow Drive storage account with the specified name and number of bytes.
        fn delete_account(&self, name: &str) -> PyResult<()> {
            self.runtime
                .block_on(self.rust_client.delete_storage_account(&try_pubkey(name)?))
                .map(|response| {
                    println!("deleted storage account {name}: {}", response.txid);
                })
                .map_err(|err| {
                    PyValueError::new_err(format!("failed to delete storage account {err:?}"))
                        .into()
                })
        }

        /// add_storage(amount, /)
        /// --
        ///
        /// Add bytes to the current storage account (set by set_account, or when creating with use_account=True)
        fn add_storage(&self, amount: u64) -> PyResult<()> {
            self.runtime.block_on(async {
                // Check if account is immutable
                if let Some(ref account) = self.current_account {
                    let storage_account = self
                        .rust_client
                        .get_storage_account(account)
                        .await
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!(
                                "unable to retrieve storage account {e:?}"
                            ))
                        })?;

                    if storage_account.is_immutable() {
                        self.rust_client
                            .add_immutable_storage(account, Byte::from(amount))
                            .await
                            .map(|response| {
                                println!("AddImmutableStorage Response: {}", response.message);
                            })
                            .map_err(|err| {
                                PyValueError::new_err(format!("failed to add storage {err:?}"))
                                    .into()
                            })
                    } else {
                        self.rust_client
                            .add_storage(account, Byte::from(amount))
                            .await
                            .map(|response| {
                                println!("AddStorage Response: {}", response.message);
                            })
                            .map_err(|err| {
                                PyValueError::new_err(format!("failed to add storage {err:?}"))
                                    .into()
                            })
                    }
                } else {
                    Err(PyRuntimeError::new_err(
                        "no storage account is set. Set one using the set_account(...) method",
                    ))
                }
            })
        }

        /// reduce_storage(amount, /)
        /// --
        ///
        /// Reduce bytes of the current storage account (set by set_account, or when creating with use_account=True)
        fn reduce_storage(&self, amount: u64) -> PyResult<()> {
            self.runtime.block_on(async {
                // Check if account is immutable
                if let Some(ref account) = self.current_account {
                    let storage_account = self
                        .rust_client
                        .get_storage_account(account)
                        .await
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!(
                                "unable to retrieve storage account {e:?}"
                            ))
                        })?;

                    let is_immutable: bool = storage_account.is_immutable();
                    let total_storage = storage_account.storage();
                    if total_storage < amount {
                        return Err(PyRuntimeError::new_err(format!("Account only has {total_storage} bytes, but you attempted to reduce by {amount}")));
                    }

                    if is_immutable {
                        Err(PyRuntimeError::new_err(
                            "Account selected is immutable. Cannot remove storage",
                        ))
                    } else {
                        self.rust_client
                            .reduce_storage(account, Byte::from(amount))
                            .await
                            .map(|response| {
                                println!("RemoveStorage Response: {}", response.message);
                            })
                            .map_err(|err| {
                                PyValueError::new_err(format!("failed to add storage {err:?}"))
                                    .into()
                            })
                    }
                } else {
                    Err(PyRuntimeError::new_err(
                        "no storage account is set. Set one using the set_account(...) method",
                    ))
                }
            })
        }

        /// upload_files(files, /)
        /// --
        ///
        /// Upload the specified files. Note that any non-Unicode characters in the file name are
        /// converted to the U+FFFD REPLACEMENT CHARACTER.
        fn upload_files(&self, files: Vec<&str>, py: Python) -> PyResult<Vec<Py<PyString>>> {
            if let Some(ref storage_account) = self.current_account {
                // Turn files provided into ShadowFiles
                let files: Vec<ShadowFile> = files
                    .into_iter()
                    .map(|file| {
                        let path: &Path = Path::new(file);
                        if let Some(name) = path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                        {
                            Ok(ShadowFile::file(name, path))
                        } else {
                            Err(PyValueError::new_err(format!(
                                "an invalid file path was provided: {}",
                                path.display()
                            )))
                        }
                    })
                    .collect::<PyResult<Vec<ShadowFile>>>()?;

                // Upload files
                let response: ShadowUploadResponse = self
                    .runtime
                    .block_on(self.rust_client.store_files(storage_account, files))
                    .map_err(|err| {
                        PyValueError::new_err(format!("failed to upload files: {err:?}"))
                    })?;

                // Alert the user of any errors
                for error in &response.upload_errors {
                    println!("failed to upload file {}: {}", &error.file, &error.error);
                }

                // Return successful uploads
                let successes = response
                    .finalized_locations
                    .iter()
                    .map(|loc| PyString::new(py, loc).into())
                    .collect();
                Ok(successes)
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, or specify one with set_account"))
            }
        }

        /// delete_files(file_urls, /)
        /// --
        ///
        /// Delete the specified files (that live at the specified urls) in the current_storage account.
        fn delete_files(&self, file_urls: Vec<String>) -> PyResult<()> {
            if let Some(ref storage_account) = self.current_account {
                self.runtime.block_on(async move {
                    tokio_scoped::scope(|scope| {
                        for url in file_urls {
                            scope.spawn(async move {
                                if let Err(err) = self
                                    .rust_client
                                    .delete_file(&storage_account, url.clone())
                                    .await
                                {
                                    println!("failed to delete file {url}: {err:?}");
                                }
                            });
                        }
                    });
                });
                Ok(())
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, or specify one with set_account"))
            }
        }

        /// list_files(/)
        /// --
        ///
        /// List all files associated with the current storage account (if account=None). If an account is provided, those files are checked instead.
        fn list_files(&self, account: Option<&str>) -> PyResult<Vec<String>> {
            let account_to_check = self
                .current_account
                .map(Result::Ok)
                .or(account.map(Pubkey::from_str));

            if let Some(Ok(ref storage_account)) = account_to_check {
                self.runtime.block_on(async move {
                    self.rust_client
                        .list_objects(storage_account)
                        .await
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!(
                                "failed to gather files for storage account: {e:?}"
                            ))
                        })
                })
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, specify one with set_account, or pass in the 'account' optional arugment"))
            }
        }

        /// get_file(/)
        /// --
        ///
        /// Retrieve the specified file if it exists in the storage account. Can also provide a url to a file (need not be in the current storage account).
        fn get_file(&self, file: &str) -> PyResult<Vec<u8>> {
            let url = if file.contains(SHDW_DRIVE_OBJECT_PREFIX) {
                file.to_string()
            } else {
                if let Some(ref storage_account) = self.current_account {
                    format!("{SHDW_DRIVE_OBJECT_PREFIX}")
                } else {
                    return Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, specify one with set_account, or pass in the 'account' optional arugment"));
                }
            };
            self.runtime.block_on(async move {
                reqwest::get(url)
                    .await
                    .map(|response| response.bytes())
                    .map_err(|e| PyRuntimeError::new_err(format!("failed to retrieve file {e:?}")))?
                    .await
                    .map(|bytes| bytes.to_vec())
                    .map_err(|e| PyRuntimeError::new_err(format!("failed to retrieve file {e:?}")))
            })
        }

        /// cancel_delete_storage(/)
        /// --
        ///
        /// If a storage account was previously requested to be deleted, this sends a request to cancel that deletion request.
        /// Sends the request for another account if it is provided. If successful, returns transaction id
        fn cancel_delete_account(&self, account: Option<&str>) -> PyResult<String> {
            let account_to_check = self
                .current_account
                .map(Result::Ok)
                .or(account.map(Pubkey::from_str));

            if let Some(Ok(ref storage_account)) = account_to_check {
                self.runtime.block_on(async move {
                    self.rust_client
                        .cancel_delete_storage_account(storage_account)
                        .await
                        .map(|response| {
                            println!("CancelDeleteAccount Response: {}", response.txid);
                            response.txid
                        })
                        .map_err(|err| {
                            PyValueError::new_err(format!("failed to add storage {err:?}")).into()
                        })
                })
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, specify one with set_account, or pass in the 'account' optional arugment"))
            }
        }

        /// make_account_immutable(skip_warning/)
        /// --
        ///
        /// Makes account immutable. NOTE: THIS IS IRREVERSIBLE!
        fn make_account_immutable(&self, skip_warning: Option<bool>) -> PyResult<()> {
            if let Some(ref storage_account) = self.current_account {
                // Warn user if they are not skipping
                if skip_warning != Some(true) {
                    println!("You are about to make {storage_account} immutable. This is a permanent, irreversible action. Proceed? [y/n]");
                    let mut user_input = String::new();
                    let _ = std::io::stdin().read_line(&mut user_input);

                    if !["yes", "y"].contains(&user_input.to_lowercase().as_ref()) {
                        println!("Did not mark account as immutable");
                        return Ok(());
                    }
                }

                self.runtime.block_on(async move {
                    self.rust_client
                        .make_storage_immutable(storage_account)
                        .await
                        .map(|response| {
                            println!("RemoveStorage Response: {}", response.message);
                        })
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!(
                                "failed to mark account as immutable: {e:?}"
                            ))
                        })
                })
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, specify one with set_account, or pass in the 'account' optional arugment"))
            }
        }

        /// claim_stake(skip_warning/)
        /// --
        ///
        /// Claims outstanding for current account (or provided optional account). Returns transaction signature if successful.
        fn claim_stake(&self, account: Option<&str>) -> PyResult<String> {
            let account_to_check = self
                .current_account
                .map(Result::Ok)
                .or(account.map(Pubkey::from_str));

            if let Some(Ok(ref storage_account)) = account_to_check {
                self.runtime.block_on(async move {
                    self.rust_client
                        .claim_stake(storage_account)
                        .await
                        .map(|response| {
                            println!("ClaimStake Response: {}", response.txid);
                            response.txid
                        })
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!(
                                "failed to claim stake for storage account: {e:?}"
                            ))
                        })
                })
            } else {
                Err(PyRuntimeError::new_err("No storage account is specified. Create one with create_account, specify one with set_account, or pass in the 'account' optional arugment"))
            }
        }

        /// set_account(account, /)
        /// --
        ///
        /// Specify an existing storage account to manage or use.
        fn set_account(&mut self, account: &str) -> PyResult<()> {
            self.current_account = Some(Pubkey::from_str(account).map_err(|err| {
                PyValueError::new_err(format!(
                    "an invalid Pubkey {} was provided: {}",
                    account, err
                ))
            })?);

            Ok(())
        }
    }

    m.add_class::<ShadowDriveClient>()?;

    Ok(())
}

// fn get_keypair_from_object(keypair: PyObject, py: Python) -> PyResult<Keypair> {
//     // Try to grab byte array
//     let bytes: [u8; 64] = keypair.call_method0(py, "to_bytes_array")?.extract(py)?;

//     // Build keypair
//     Ok(Keypair::from_bytes(&bytes)
//         .expect("should not fail since we have valid 64 byte array at this point"))
// }

fn extract_commitment(commitment: &str) -> PyResult<CommitmentConfig> {
    match commitment.to_lowercase().as_ref() {
        "processed" => Ok(CommitmentConfig::processed()),
        "confirmed" => Ok(CommitmentConfig::confirmed()),
        "finalized" => Ok(CommitmentConfig::finalized()),
        _ => Err(PyValueError::new_err(
            "the only acceptable commitment values are 'processed', 'confirmed', and 'finalized'.",
        )),
    }
}

fn check_current_account(
    account: Option<&str>,
    rust_client: &ShadowDriveRustClient<Keypair>,
    runtime: &Runtime,
) -> Option<Pubkey> {
    if let Some(acct) = account {
        match Pubkey::from_str(&acct) {
            // if we got a valid pubkey, check that it is a valid storage account
            Ok(key) => runtime
                .block_on(rust_client.get_storage_account(&key))
                .map(|_| key)
                .map_err(|err| {
                    println!("invalid account pubkey provided: {err:?}");
                })
                .ok(),

            // Otherwise, return None
            Err(err) => {
                println!("invalid account pubkey provided: {err:?}");
                None
            }
        }
    } else {
        None
    }
}

fn try_pubkey(key: &str) -> PyResult<Pubkey> {
    Pubkey::from_str(key).map_err(|err| {
        PyValueError::new_err(format!("an invalid Pubkey {} was provided: {}", key, err))
    })
}
