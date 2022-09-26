# Shadow Drive Rust

Rust SDK for [GenesysGo's Shadow Drive](https://shdw.genesysgo.com/shadow-infrastructure-overview/shadow-drive-overview), a decentralized storage network.

Available on [crates.io](https://crates.io/crates/shadow-drive-rust).

## Install

Add the crate to your `Cargo.toml`.

```toml
shadow-drive-rust = "0.4.0"
```

### Examples

```rust
    //init tracing.rs subscriber
    tracing_subscriber::fmt()
        .with_env_filter("off,shadow_drive_rust=debug")
        .init();

    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    //derive the storage account pubkey
    let pubkey = keypair.pubkey();
    let (storage_account_key, _) =
        shadow_drive_rust::derived_addresses::storage_account(&pubkey, 0);

    // read files in directory
    let dir = tokio::fs::read_dir("multiple_uploads")
    .await
    .expect("failed to read multiple uploads dir");

    // create Vec of ShadowFile structs for upload
    let mut files = tokio_stream::wrappers::ReadDirStream::new(dir)
        .filter(Result::is_ok)
        .and_then(|entry| async move {
            Ok(ShadowFile::file(
                entry
                    .file_name()
                    .into_string()
                    .expect("failed to convert os string to regular string"),
                entry.path(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()
        .await
        .expect("failed to create shdw files for dir");

    // Bytes are also supported
    files.push(ShadowFile::bytes(
        String::from("buf.txt"),
        &b"this is a buf test"[..],
    ));

    // kick off upload
    let upload_results = shdw_drive_client
        .upload_multiple_files(&storage_account_key, files)
        .await
        .expect("failed to upload files");

    //profit
    println!("upload results: {:#?}", upload_results);
```

More examples can be found in the [`example`](https://github.com/VegetarianOrc/shadow-drive-rust/tree/main/example) directory.
