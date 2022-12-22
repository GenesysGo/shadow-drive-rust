# Changelog

## [1.5.0]
### Deprecated
- Removed on-chain `File` accounts. Since this is no longer used, removed `store_file`, `edit_file`, `request_delete_file`, `unmark_delete_file`, `delete_file` from `lib.rs` and moved respective source code for handlers to `src/instructions/archive/`.
- Removed on-chain tracking of shades staked for storage and of mutable fees. Uploader server must now verify that users have enough storage available before the user decreases storage via `decrease_storage`. May also want to provide an ix for the uploader server to figure out max reduction possible as was dealt with previously when the `Option<u64>` was `None`. Note that this renders many fields in the v1 struct null/useless.


### Changes
- Added a `StorageAccountV2` struct.
- Added a set of instructions specifically for `StorageAccountV2`. 
- Added several traits and implemented them for `Context<_V1>` and `Context<_V2>` for instructions so that each struct goes through the same instruction handler.
- Marking an account as immutable no longer maximally reduces the total storage of a storage account. It also no longer disables `increase_storage` and no longer restricts file uploads. In other words, it becomes an (add + expand)-only storage solution, which preserves the desired immutability for existing data when flagged.
