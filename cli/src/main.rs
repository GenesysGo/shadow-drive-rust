use anyhow::anyhow;
use clap::{IntoApp, Parser};
use shadow_drive_cli::Opts;
use shadow_rpc_auth::{authenticate, parse_account_id_from_url};
use solana_clap_v3_utils::keypair::keypair_from_path;

pub const GENESYSGO_AUTH_KEYWORD: &str = "genesysgo";

const NO_CONFIG_FILE: &str = "\
Cannot find a config file. You likely do not have the official Solana CLI installed.
Either install the Solana CLI or place a configuration file at ~/.config/solana/cli/config.yml
See https://docs.solana.com/cli/install-solana-cli-tools for installation details.
";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CLI Parse
    let opts = Opts::parse();

    // Get signer string from either an argument or the Solana CLI config file
    let app = Opts::into_app();
    let matches = app.get_matches();
    let config = {
        let config_file = solana_cli_config::CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow!("unable to determine a config file path on this OS or user"))?;
        solana_cli_config::Config::load(&config_file).map_err(|_| anyhow!(NO_CONFIG_FILE))
    }?;
    let keypath = opts
        .cfg_override
        .keypair
        .unwrap_or_else(|| config.keypair_path.clone());
    let signer = keypair_from_path(
        &matches,
        shellexpand::tilde(&keypath).as_ref(),
        "keypair",
        false,
    )
    .unwrap();
    // TODO: refactor to a single keypair after https://github.com/solana-labs/solana/pull/32181
    let signer_2 = keypair_from_path(
        &matches,
        shellexpand::tilde(&keypath).as_ref(),
        "keypair",
        false,
    )
    .unwrap();

    // Resolve the RPC URL from either a command-line arg or the Solana CLI config file.
    let url = opts.cfg_override.url.unwrap_or(config.json_rpc_url);

    // Possibly perform a sign-in operation
    let mut auth: Option<String> = opts.cfg_override.auth.clone();
    if opts.cfg_override.auth == Some(GENESYSGO_AUTH_KEYWORD.to_string()) {
        let account_id = parse_account_id_from_url(url.to_string())?;
        let token = authenticate(&signer, &account_id).await?;
        auth = Some(token)
    };

    opts.command
        .process(
            &signer,
            signer_2,
            &url,
            opts.cfg_override.skip_confirm,
            auth,
        )
        .await?;
    Ok(())
}
