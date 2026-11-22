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
| Collection deletion originally left records visible and ignored delete deltas during save. | [`Collection::delete`](../../src/core/document/collection/mod.rs#L187) now removes the directory entry, and [save](../../src/core/document/collection/mod.rs#L226) patches neighboring links. | Resolved | Remaining deletion cases are tracked below. |
| Deleting an unsaved record retained its create delta and dropped unrelated pending updates. | The [retain predicate](../../src/core/document/collection/mod.rs#L194) now keeps entries for other IDs. | Resolved | The pending create is canceled. |
| Saving deletions could leave a stale count or append through an invalid tail offset. | The [append branch](../../src/core/document/collection/mod.rs#L254) checks the post-delete count, and [save](../../src/core/document/collection/mod.rs#L292) updates the cached count. | Resolved | Consecutive successful saves use the updated count. |
| A failed collection header write could discard pending updates. | [Save](../../src/core/document/collection/mod.rs#L292) now writes the header before assigning the cached count and clearing pending updates and tracked records. | Resolved | Pending state is retained when a header write fails. Replay safety is tracked below. |
| Retrying a partial collection save can append the same record twice. | The [append loop](../../src/core/document/collection/mod.rs#L262) writes records and advances the in-memory tail before the header writes. If an existing collection appends one record and the header write fails, retrying can link a second copy after the first while counting the pending create once. | Open | |
| Saved records could disappear from queries until the document was reopened. | [Save](../../src/core/document/collection/mod.rs#L287) now updates newly written records to `OnDisk` before clearing tracked payloads. | Resolved | The directory points to the saved record. |
| Pitaka's locked GPUI backend supports opt-in hot reload for desktop view rendering. | [`Cargo.lock`](../../Cargo.lock) resolves GPUI Kit 0.9.1 to `gpui-pre` 0.3.12. The backend defines `hot-reload`, but Kit 0.9.1 does not forward it, and `cargo tree --offline -e features -i gpui-pre` confirms it is inactive in Pitaka. [GPUI hot reload integration](https://github.com/zed-industries/zed/pull/65738) | Open | |

## Devlogs

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 09:09 AM +08:00 | Checked the locked GPUI stack for live UI editing. A future setup can enable the backend feature through a direct dependency and test a home view edit under Dioxus CLI on macOS. |

</details>
