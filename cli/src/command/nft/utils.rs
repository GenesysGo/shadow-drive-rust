use serde_json::Value;

/// This function ensures the contents of a JSON file are compliant with the Metaplex Standard
/// which we define as a JSON with the non-null values for the following fields:
///
/// 1) `name`:  Name of the asset.
/// 2) `symbol`: Symbol of the asset.
/// 3) `description`: Description of the asset.
/// 4) `image`: URI pointing to the asset's logo.
/// 5) `animation_url`: URI pointing to the asset's animation.
/// 6) `external_url`: URI pointing to an external URL defining the asset — e.g. the game's main site.
/// 7) `attributes`: Array of attributes defining the characteristics of the asset.
///    a) `trait_type`: The type of attribute.
///    b) `value`: The value for that attribute.
///
/// This is taken from https://docs.metaplex.com/programs/token-metadata/token-standard and reformatted.
///
/// The function simply checks whether the fields are non-null
pub(crate) fn validate_json_compliance(json: &Value) -> bool {
    let has_name = json.get("name").is_some();
    let has_symbol = json.get("symbol").is_some();
    let has_description = json.get("description").is_some();
    let has_image = json.get("image").is_some();
    let has_animation_url = json.get("animation_url").is_some();
    let has_external_url = json.get("external_url").is_some();
    let has_attributes = json.get("attributes").is_some();

    has_name
        & has_symbol
        & has_description
        & has_image
        & has_animation_url
        & has_external_url
        & has_attributes
}

use std::{error::Error, str::FromStr};

use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::read_keypair_file, signer::Signer, transaction::VersionedTransaction,
};

// A working swap example which the swap lib is based on
// START EXAMPLE
//
// #[tokio::main]
// async fn main() {
//     const ONE_SHDW: u64 = 1_000_000_000;
//     let kp = read_keypair_file("test.json").unwrap();

//     let quote = Value::from_str(&quote(ONE_SHDW).await.unwrap()).unwrap();
//     println!("{}", serde_json::to_string_pretty(&quote).unwrap());
//     println!("{}", quote["data"][0]["priceImpactPct"]);

//     let swap_tx = swap(&quote["data"], kp.pubkey()).await.unwrap();
//     #[allow(deprecated)]
//     let tx_bytes = base64::decode(swap_tx["swapTransaction"].as_str().unwrap()).unwrap();
//     let mut tx: VersionedTransaction = bincode::deserialize(&tx_bytes).unwrap();

//     let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
//     tx.message
//         .set_recent_blockhash(client.get_latest_blockhash().await.unwrap());
//     tx.signatures[0] = kp.sign_message(&tx.message.serialize());
//     client.send_and_confirm_transaction(&tx).await.unwrap();

//     println!("tx = {tx:#?}");
// }

// async fn quote(amount: u64) -> Result<String, Box<dyn Error>> {
//     const SHDW_MINT: &'static str = "SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y";
//     const SOL_MINT: &'static str = "So11111111111111111111111111111111111111112";
//     const SLIPPAGE_BPS: u16 = 5;

//     let url = format!(
//         "https://quote-api.jup.ag/v4/quote?inputMint={}&outputMint={}&amount={}&slippageBps={SLIPPAGE_BPS}&swapMode=ExactOut",
//         SOL_MINT, SHDW_MINT, amount
//     );

//     let response = reqwest::Client::new()
//         .get(&url)
//         .header("accept", "application/json")
//         .send()
//         .await?;

//     let body = response.text().await?;

//     Ok(body)
// }

// async fn swap(quote: &Value, user: Pubkey) -> Result<Value, Box<dyn Error>> {
//     let url = "https://quote-api.jup.ag/v4/swap";

//     let request_body = json!({
//         "route": dbg!(&quote[0]),
//         "userPublicKey": user.to_string(),
//         "wrapUnwrapSOL": true,
//         // "feeAccount": fee_account // in case we ever want to charge a fee
//     });

//     let client = reqwest::Client::new();
//     let response = client
//         .post(url)
//         .header("Content-Type", "application/json")
//         .json(&request_body)
//         .send()
//         .await?;

//     let body = response.text().await?;

//     Ok(serde_json::Value::from_str(&body).unwrap())
// }
//
// END EXAMPLE

pub(crate) async fn swap_sol_for_shdw_tx(
    shades: u64,
    user: Pubkey,
) -> anyhow::Result<VersionedTransaction> {
    // First we get the best route/quote
    let Ok(quote) = Value::from_str(&quote_sol_to_shdw(shades).await?) else {
        return Err(anyhow::Error::msg("Failed to parse jup.ag quote response as json"))
    };

    // Then request the transaction for this swap
    let request_body = json!({
        "route": dbg!(&quote[0]),
        "userPublicKey": user.to_string(),
        "wrapUnwrapSOL": true,
        // "feeAccount": fee_account // in case we ever want to charge a fee
    });
    let client = reqwest::Client::new();
    let response = client
        .post("https://quote-api.jup.ag/v4/swap")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    // Parse response as json
    let Ok(body) = serde_json::Value::from_str(&response.text().await?) else {
        return Err(anyhow::Error::msg("Failed to parse jup.ag swap_tx response as json"))
    };

    // Deserialize response into VersionedTransaction
    let Some(Some(tx_body))= body.get("swapTransaction").map(|b| b.as_str()) else {
        return Err(anyhow::Error::msg("Unexpected response from jup.ag swap_tx endpoint"))
    };
    #[allow(deprecated)]
    let Ok(Ok(transaction)) = base64::decode(tx_body).map(|bytes| bincode::deserialize(&bytes)) else {
        return Err(anyhow::Error::msg("Invalid base64 encoding from jup.ag swap_tx endpoint"))
    };

    Ok(transaction)
}

pub(crate) const SHDW_MINT: &'static str = "SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y";
pub(crate) const SOL_MINT: &'static str = "So11111111111111111111111111111111111111112";
pub(crate) const SHDW_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([
    6, 121, 219, 1, 206, 42, 132, 247, 28, 19, 158, 124, 153, 66, 246, 218, 59, 51, 31, 222, 195,
    49, 157, 2, 248, 153, 235, 167, 1, 52, 115, 126,
]);
pub(crate) const SOL_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([
    6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196, 57, 220, 26,
    235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1,
]);

async fn quote_sol_to_shdw(shades: u64) -> anyhow::Result<String> {
    const SLIPPAGE_BPS: u16 = 5;

    let url = format!(
        "https://quote-api.jup.ag/v4/quote?inputMint={}&outputMint={}&amount={}&slippageBps={SLIPPAGE_BPS}&swapMode=ExactOut",
        SOL_MINT, SHDW_MINT, shades
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?;

    let body = response.text().await?;

    Ok(body)
}
