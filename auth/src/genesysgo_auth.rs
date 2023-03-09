use anyhow::anyhow;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use solana_sdk::{bs58, signature::Signer};

const SIGNIN_MSG: &str = "Sign in to GenesysGo Shadow Platform.";
const PORTAL_SIGNIN_URL: &str = "https://portal.genesysgo.net/api/signin";
const RPC_SIGNIN_URL: &str = "https://portal.genesysgo.net/api/premium/token";

/// The request body for GenesysGo Portal/Network Authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuth {
    /// Signed and base-58 encoded SIGNIN_MSG
    message: String,
    /// Base58 pubkey
    signer: String,
}

/// The response object for GenesysGo Network/Portal authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct GenesysGoAuthResponse {
    /// Bearer token
    pub token: String,
    /// Extra user information from the GenesysGo network.
    pub user: GenesysGoUser,
}

/// User data about the signed-in account.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesysGoUser {
    pub id: u64,
    pub public_key: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A token returned on successful GenesysGo RPC authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Bearer token
    pub token: String,
}

/// Acquire a bearer token to a GenesysGo Premium RPC account. The signer must be whitelisted.
///
/// This function makes two requests. Its first request acquires a GenesysGo Network auth token,
/// which it then uses to acquire the RPC auth token.
pub async fn authenticate(signer: &dyn Signer, account_id: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let resp = genesysgo_portal_auth(signer, &client).await?;
    let resp = genesysgo_rpc_auth(account_id, &resp.token, &client).await?;
    Ok(resp.token)
}

/// Authenticate to the GenesysGo Portal/Network. If your ultimate aim is
/// to get an auth token for RPC, this is the first step.
pub async fn genesysgo_portal_auth(
    signer: &dyn Signer,
    client: &reqwest::Client,
) -> anyhow::Result<GenesysGoAuthResponse> {
    let signature = signer.sign_message(SIGNIN_MSG.as_bytes());
    let body = GenesysGoAuth {
        message: bs58::encode(signature.as_ref()).into_string(),
        signer: signer.pubkey().to_string(),
    };
    let resp = client
        .post(Url::parse(PORTAL_SIGNIN_URL)?)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?;
    let auth_resp: GenesysGoAuthResponse = serde_json::from_str(&resp.text().await?)?;
    Ok(auth_resp)
}

/// Using the bearer token acquired from [genesysgo_portal_auth],
/// acquire an RPC auth token for a GenesysGo Premium RPC account based on its Account ID.
pub async fn genesysgo_rpc_auth(
    account_id: &str,
    step_1_auth_token: &str,
    client: &reqwest::Client,
) -> anyhow::Result<TokenResponse> {
    let step2_url = RPC_SIGNIN_URL.to_owned() + "/" + account_id;
    let bearer_token = format!("Bearer {}", step_1_auth_token);
    let resp = client
        .post(Url::parse(&step2_url)?)
        .header("Content-Type", "application/json")
        .header("Authorization", &bearer_token)
        .send()
        .await?;
    let resp: TokenResponse = serde_json::from_str(&resp.text().await?)?;
    Ok(resp)
}

/// If you only have a GenesysGo RPC URL, this will parse out
/// the account ID necessary to perform a sign-in.
pub fn parse_account_id_from_url(genesysgo_url: String) -> anyhow::Result<String> {
    if !genesysgo_url.contains("genesysgo") {
        return Err(anyhow!("Not a genesysgo URL, cannot infer Account ID"));
    }
    let pieces = genesysgo_url.split("/");
    let last = pieces
        .last()
        .ok_or(anyhow!("Could not parse genesysgo url: {}", &genesysgo_url))?;
    Ok(last.to_string())
}
