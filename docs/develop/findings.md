[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Findings

# Findings

This acts as both official documentation and devlogs.

## Contents

- [Tracked findings](#tracked-findings)

## Tracked findings

The developer and agent can both add verified findings. Include the observed behavior and a code reference or reproduction steps. Mark entries `Open` when added and update the same entry with the fix when resolved.

| Finding | Evidence | Status | Resolution |
| --- | --- | --- | --- |
| Saved records can disappear from queries until the document is reopened. | [`Collection::add`](../../src/core/document/collection/mod.rs#L173) sets the directory locator to `InMemory`. After writing the record, [`write_to_file`](../../src/core/document/collection/mod.rs#L225) clears `tracked` without changing the locator to `OnDisk`. [`Collection::query`](../../src/core/document/collection/mod.rs#L154) then looks for the record in `tracked` and skips it. Reopening rebuilds the locator from the persisted record. | Open | |
