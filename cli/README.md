## Shadow Drive CLI (Rust Version)

This is a CLI for Shadow Drive written using the [Shadow Drive Rust](https://github.com/VegetarianOrc/shadow-drive-rust) crate. It is largely a `clap` wrapper around that crate.

Although there is already a CLI for Shadow Drive written in Typescript, I wanted something
with identical interface to the official Solana CLI `-k/--keypair` and `-u/--url` arguments.
You can pass all the same signer types ("prompt", "stdin", etc), but be aware that Ledger still does not support
general message signing, and you therefore cannot perform most Shadow Network operations
using a hardware wallet yet.

This is an opinionated CLI, choosing V2 storage accounts where applicable.

The CLI also works with authenticated GenesysGo Premium RPC Endpoints. See
the `--auth` flag for more details.

## Build
Build the binary like a standard Rust crate.
```
cargo build
```

## Execution
The CLI looks for a Solana CLI config file. Configure your signing key there using the usual `solana config set -k <SIGNER>`, or pass one in using the `-k/--keypair` argument. If no `-k/--keypair` argument is used and no config file is found, this CLI defaults to the same keypair path as the Solana CLI's default, located at `.config/solana/id.json`.

For further usage details, there is extensive help text built into the binary.
```
$ target/debug/shadow-drive-cli --help
```

## TODO
- Better Error Handling
- Testing
