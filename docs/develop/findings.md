[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Findings

# Findings

This acts as both official documentation and devlogs.

## Contents

- [Tracked findings](#tracked-findings)
- [Devlogs](#devlogs)

## Tracked findings

The developer and agent can both add verified findings. Include the observed behavior and a code reference or reproduction steps. Mark entries `Open` when added and update the same entry with the fix when resolved.

| Finding | Evidence | Status | Resolution |
| --- | --- | --- | --- |
| The account lookup records an ID filter that the query collector does not apply. | [The builder](../../src/core/vault/collection/query.rs) adds the filter to `plan.filters`, while [the collector](../../src/core/vault/collection/mod.rs) selects candidates using only `plan.indexed_filters`. | Open | |
| Querying documented an outdated index registration method and value type. | [Collection](../../src/core/vault/collection/mod.rs) exposes `add_index`, and [IndexValue](../../src/core/vault/collection/index/value.rs) supports `U8` and `U64`. | Resolved | Updated the [collection index documentation](querying.md#collection-indexes). |
| Editing an indexed field can leave the record ID under its old key. | [Collection updates](../../src/core/vault/collection/mod.rs#L160) replace the pending payload without updating index buckets. Indexed queries select IDs from those buckets without checking the payload against the indexed value. | Open | |
| Creating a vault could overwrite the vault header with root data. | A create, save, and reopen check preserved the vault header and returned the saved account and transaction. | Resolved | Root creation assigns its header offset and the root writer reserves the header space before appending records. |
| A missing catalog entry could leave a managed collection with header position zero. | A vault with only an Accounts catalog entry opened successfully, saved a new transaction, and returned that transaction after reopening. | Resolved | The root saves the catalog first and resolves the child header position before saving the child. |
| Managed collection saves could leave catalog entries stale and overwrite record data. | Fresh vaults passed save and reopen checks for one account and one transaction, repeated saves, and two accounts added in one save. | Resolved | The root saves each managed collection using the position of its embedded header. Existing files with prior corruption were outside these checks. |
| The accounts page referenced the removed direct account and transaction fields. | `cargo check --offline` previously reported three E0609 errors in [the accounts page](../../src/views/main/pages/accounts.rs). | Resolved | The page uses the vault's typed collection accessors. The check passes. |
| Creating a vault reached an unimplemented managed collection writer. | [Vault creation](../../src/core/vault/mod.rs) adds managed collections and writes the root. | Resolved | [The managed payload writer](../../src/core/vault/collection/managed.rs) writes the embedded 40-byte header. |
| The vault rename left documentation links pointing to the old module. | Code Organization and Findings referenced `src/core/document`, and Querying used the old `PitakaDocument` type name. | Resolved | Uses `src/core/vault` and `PitakaVault`. |
| Collection deletion originally left records visible and ignored delete deltas during save. | [`Collection::delete`](../../src/core/vault/collection/mod.rs#L187) now removes the directory entry, and [save](../../src/core/vault/collection/mod.rs#L226) patches neighboring links. | Resolved | Remaining deletion cases are tracked below. |
| Deleting an unsaved record retained its create delta and dropped unrelated pending updates. | The [retain predicate](../../src/core/vault/collection/mod.rs#L194) now keeps entries for other IDs. | Resolved | The pending create is canceled. |
| Saving deletions could leave a stale count or append through an invalid tail offset. | The [append branch](../../src/core/vault/collection/mod.rs#L254) checks the post-delete count, and [save](../../src/core/vault/collection/mod.rs#L292) updates the cached count. | Resolved | Consecutive successful saves use the updated count. |
| A failed collection header write could discard pending updates. | [Save](../../src/core/vault/collection/mod.rs#L292) now writes the header before assigning the cached count and clearing pending updates and tracked records. | Resolved | Pending state is retained when a header write fails. Replay safety is tracked below. |
| Retrying a partial collection save can append the same record twice. | The [append loop](../../src/core/vault/collection/mod.rs#L262) writes records and advances the in-memory tail before the header writes. If an existing collection appends one record and the header write fails, retrying can link a second copy after the first while counting the pending create once. | Open | |
| Saved records could disappear from queries until the document was reopened. | [Save](../../src/core/vault/collection/mod.rs#L287) now updates newly written records to `OnDisk` before clearing tracked payloads. | Resolved | The directory points to the saved record. |
| Pitaka enables GPUI hot reload, and an Accounts page render panicked during a development session. | `cargo tree --offline -e features -i gpui-pre` confirms [Pitaka enables the feature directly](../../Cargo.toml). The macOS log shows GPUI 0.3.12 failing to downcast the stored `AccountsPage` view before invoking its renderer. | Open | A mismatch between a live entity and the renderer after a patch is a possible cause. A clean-process comparison is pending. |

## Devlogs

<details>
<summary><strong>Nov. 25, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 08:36 AM +08:00 | Confirmed the GPUI hot-reload feature is active and traced the reported Accounts page panic to the erased view downcast. The log ends with a separate SIGKILL after a manual rebuild. |

</details>

<details>
<summary><strong>Nov. 23, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 10:43 AM +08:00 | Verified the current root save flow with temporary tests covering a save and reopen, a repeated save, a batch append, and recovery of a missing managed collection header. |

</details>

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 09:09 AM +08:00 | Checked the locked GPUI stack for live UI editing. A future setup can enable the backend feature through a direct dependency and test a home view edit under Dioxus CLI on macOS. |

</details>
