use shadow_drive_sdk::{models::ShadowFile, ShadowDriveClient};
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");

    let v1_pubkey = Pubkey::from_str("GSvvRguVTtSayF5zLQPLVTJQHQ6Fqu1Z3HSRMP8AHY9K").unwrap();
    let v2_pubkey = Pubkey::from_str("2cvgcqfmMg9ioFtNf57ZqCNbuWDfB8ZSzromLS8Kkb7q").unwrap();

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    //add a file
    let v1_upload_reponse = shdw_drive_client
        .store_files(
            &v1_pubkey,
            vec![ShadowFile::file(
                String::from("example.png"),
                "multiple_uploads/0.txt",
            )],
        )
        .await
        .expect("failed to upload v1 file");
    println!("Upload complete {:?}", v1_upload_reponse);

    let v2_upload_reponse = shdw_drive_client
        .store_files(
            &v2_pubkey,
            vec![ShadowFile::file(
                String::from("example.png"),
                "multiple_uploads/0.txt",
            )],
        )
        .await
        .expect("failed to upload v2 file");

    println!("Upload complete {:?}", v2_upload_reponse);

    let v1_url = String::from(
        "https://shdw-drive.genesysgo.net/GSvvRguVTtSayF5zLQPLVTJQHQ6Fqu1Z3HSRMP8AHY9K/example.png",
    );
    let v2_url = String::from(
        "https://shdw-drive.genesysgo.net/2cvgcqfmMg9ioFtNf57ZqCNbuWDfB8ZSzromLS8Kkb7q/example.png",
    );

    //delete file
    let v1_delete_file_response = shdw_drive_client
        .delete_file(&v1_pubkey, v1_url)
        .await
        .expect("failed to delete file");
    println!("Delete file complete {:?}", v1_delete_file_response);

    let v2_delete_file_response = shdw_drive_client
        .delete_file(&v2_pubkey, v2_url)
        .await
        .expect("failed to delete file");
    println!("Delete file complete {:?}", v2_delete_file_response);
}
