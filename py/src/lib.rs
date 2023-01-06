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
use shadow_drive_sdk::models::{ShadowFile, ShadowUploadResponse};
use shadow_drive_sdk::{
    Byte, CommitmentConfig, Keypair, Pubkey, RpcClient, ShadowDriveClient as ShadowDriveRustClient,
    Signer,
};
use tokio::runtime::{Builder, Runtime};

/// A Python module implemented in Rust.
#[pymodule]
fn shadow_drive(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add Solana Mainnet-Beta RPC endpoint
    const SOLANA_MAINNET_BETA: &'static str = "https://api.mainnet-beta.solana.com";
    m.add("SOLANA_MAINNET_BETA", SOLANA_MAINNET_BETA)?;

    // #[cfg(debug_assertions)]
    // const WHITELIST_RPC: &'static str = "http://145.40.74.211:8899";
    // #[cfg(debug_assertions)]
    // m.add("WHITELIST_RPC", WHITELIST_RPC)?;

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
        fn new(
            keypair: PyObject,
            account: Option<&str>,
            py: Python,
        ) -> PyResult<ShadowDriveClient> {
            let rust_client = Arc::new(ShadowDriveRustClient::new(
                get_keypair_from_object(keypair, py)?,
                // RpcClient::new(
                //     // #[cfg(not(debug_assertions))]
                SOLANA_MAINNET_BETA.to_string(),
                //     // #[cfg(debug_assertions)]
                //     // WHITELIST_RPC.to_string(),
                // ),
            ));
            let runtime = Builder::new_multi_thread()
                // .worker_threads(2)
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
            keypair: PyObject,
            commitment: &str,
            account: Option<&str>,
            py: Python,
        ) -> PyResult<ShadowDriveClient> {
            // Extract commitment
            let commitment_config = extract_commitment(commitment)?;

            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                get_keypair_from_object(keypair, py)?,
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
            keypair: PyObject,
            rpc: &str,
            account: Option<&str>,
            py: Python,
        ) -> PyResult<ShadowDriveClient> {
            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                get_keypair_from_object(keypair, py)?,
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
            keypair: PyObject,
            commitment: &str,
            rpc: &str,
            account: Option<&str>,
            py: Python,
        ) -> PyResult<ShadowDriveClient> {
            // Extract commitment
            let commitment_config = extract_commitment(commitment)?;

            let rust_client = Arc::new(ShadowDriveRustClient::new_with_rpc(
                get_keypair_from_object(keypair, py)?,
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
            &self,
            name: &str,
            size: u64,
            py: Python,
        ) -> PyResult<(Py<PyString>, Py<PyString>)> {
            self.runtime.block_on(async move {
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
            })
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

    #[cfg(debug_assertions)]
    #[pyfunction]
    fn print_pubkey(keypair: PyObject, py: Python) -> Py<PyString> {
        let keypair = get_keypair_from_object(keypair, py).unwrap();
        PyString::new(py, &keypair.pubkey().to_string()).into()
    }
    #[cfg(debug_assertions)]
    m.add_function(wrap_pyfunction!(print_pubkey, m)?)?;

    #[cfg(debug_assertions)]
    {
        use pyo3::types::PyList;
        #[pyfunction]
        fn sign_message(keypair: PyObject, message: Py<PyList>, py: Python) -> Py<PyString> {
            let keypair: Keypair = get_keypair_from_object(keypair, py).unwrap();
            let sig = keypair.sign_message(message.extract::<Vec<u8>>(py).unwrap().as_ref());
            PyString::new(py, &sig.to_string()).into()
        }
        #[cfg(debug_assertions)]
        m.add_function(wrap_pyfunction!(sign_message, m)?)?;
    }

    Ok(())
}

fn get_keypair_from_object(keypair: PyObject, py: Python) -> PyResult<Keypair> {
    // Try to grab byte array
    let bytes: [u8; 64] = keypair.call_method0(py, "to_bytes_array")?.extract(py)?;

    // Build keypair
    Ok(Keypair::from_bytes(&bytes)
        .expect("should not fail since we have valid 64 byte array at this point"))
}

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
