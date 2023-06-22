use inquire::validator::Validation;
use serde_json::json;
use serde_json::Value;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use std::str::FromStr;

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
        // "feeAccount": fee_account // leaving in very unlikely case we ever want to charge a fee
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

pub(crate) fn pubkey_validator(
    input: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    // Check for valid pubkey
    if Pubkey::from_str(input).is_ok() {
        Ok(Validation::Valid)
    } else {
        Ok(Validation::Invalid("Invalid Pubkey".into()))
    }
}

pub(crate) fn validate_and_convert_to_half_percent(input: &str) -> Result<u8, &'static str> {
    // Removing possible percent sign from input
    let input = input.trim().trim_end_matches('%');

    // Try to parse input into a floating point number
    let value = input.parse::<f64>();

    match value {
        Ok(v) => {
            // Checking if value is positive and half or whole number
            if v < 0.0 {
                Err("Value must be positive.")
            } else if (2.0 * v).fract() != 0.0 {
                Err("Value must be a whole or half number.")
            } else {
                // Multiplying value by 2 to convert to half percentages and round to closest integer
                Ok((2.0 * v).round() as u8)
            }
        }
        Err(_) => Err("Invalid input, not a number."),
    }
}

#[test]
fn test_validate_and_convert_to_half_percent() {
    assert_eq!(validate_and_convert_to_half_percent("1"), Ok(2));
    assert_eq!(validate_and_convert_to_half_percent("1%"), Ok(2));
    assert_eq!(validate_and_convert_to_half_percent("1.5"), Ok(3));
    assert_eq!(validate_and_convert_to_half_percent("1.5%"), Ok(3));
    assert_eq!(validate_and_convert_to_half_percent("2"), Ok(4));
    assert_eq!(validate_and_convert_to_half_percent("2%"), Ok(4));
    assert_eq!(validate_and_convert_to_half_percent("2.0"), Ok(4));
    assert_eq!(validate_and_convert_to_half_percent("2.0%"), Ok(4));
    assert_eq!(validate_and_convert_to_half_percent("2.5"), Ok(5));
    assert_eq!(validate_and_convert_to_half_percent("2.5%"), Ok(5));

    assert_eq!(
        validate_and_convert_to_half_percent("2.4"),
        Err("Value must be a whole or half number.")
    );
    assert_eq!(
        validate_and_convert_to_half_percent("-1"),
        Err("Value must be positive.")
    );
    assert_eq!(
        validate_and_convert_to_half_percent("-1.5"),
        Err("Value must be positive.")
    );
    assert_eq!(
        validate_and_convert_to_half_percent("not a number"),
        Err("Invalid input, not a number.")
    );
    assert_eq!(
        validate_and_convert_to_half_percent(""),
        Err("Invalid input, not a number.")
    );
}
