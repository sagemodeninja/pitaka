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
| Saved records can disappear from queries until the document is reopened. | [`Collection::add`](../../src/core/document/collection/mod.rs#L173) sets the directory locator to `InMemory`. After writing the record, [`write_to_file`](../../src/core/document/collection/mod.rs#L225) clears `tracked` without changing the locator to `OnDisk`. [`Collection::query`](../../src/core/document/collection/mod.rs#L154) then looks for the record in `tracked` and skips it. Reopening rebuilds the locator from the persisted record. | Open | |
| Pitaka's locked GPUI backend supports opt-in hot reload for desktop view rendering. | [`Cargo.lock`](../../Cargo.lock) resolves GPUI Kit 0.9.1 to `gpui-pre` 0.3.12. The backend defines `hot-reload`, but Kit 0.9.1 does not forward it, and `cargo tree --offline -e features -i gpui-pre` confirms it is inactive in Pitaka. [GPUI hot reload integration](https://github.com/zed-industries/zed/pull/65738) | Open | |

## Devlogs

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 09:09 AM +08:00 | Checked the locked GPUI stack for live UI editing. A future setup can enable the backend feature through a direct dependency and test a home view edit under Dioxus CLI on macOS. |

</details>
