[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> File Layout

# File Layout

This acts as both official documentation and devlogs.

> **Warning:** This format is unstable. Any update may make existing saves unreadable. No migration or recovery tools are provided, and compatibility fixes are not planned at this point of the development.

## Contents

- [Format Rules](#format-rules)
- [File Header](#file-header)
- [Collections](#collections)
  - [Root Collection](#root-collection)
  - [Managed Collections](#managed-collections)
- [Records](#records)
  - [Root Collection](#root-collection-payload)
  - [Account](#account-payload)
  - [Transaction](#transaction-payload)
- [Structures](#structures)
  - [Length-Prefixed String](#length-prefixed-string)
- [Devlogs](#devlogs)

## Format Rules

| Rule | Specification |
| --- | --- |
| Units and types | All sizes and offsets are measured in bytes. The Type and Size columns describe the on-disk encoding and serialized width. |
| Byte order | All integers use little-endian byte order. |
| Table offsets | Each Offset column is relative to the start of the structure described by that table. |
| Stored offsets | Offset fields store absolute `u64` byte positions from the start of the file. A value of `0` means there is no target. |
| Reserved bytes | All reserved bytes must be zero. |
| Timestamps | Timestamps are signed `i64` milliseconds since the Unix epoch (1970-01-01 00:00:00 UTC). |

## File Header

80 bytes at the start of the file.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 4 | Magic bytes | `[u8; 4]` | `b"PTKA"`. |
| 4 | 2 | Format version | `u16` | Reject unsupported versions. |
| 6 | 68 | Display name | `u32` + `[u8; 64]` | Document name shown in the UI. The prefix stores the name's length in UTF-8 bytes, up to 64. Zero-pad the unused bytes in the field. |
| 74 | 6 | Reserved | `[u8; 6]` | Future header fields. |

## Collections

A collection is a table with a 40-byte header. The root collection and managed collections share this header layout.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 1 | Collection type | `u8` | The type of collection. `0` = `Root Collection`. |
| 1 | 8 | Record count | `u64` | Persisted record count. |
| 9 | 8 | Next record ID | `u64` | Next record ID; never reduced when records are deleted. |
| 17 | 8 | First record offset | `u64` | First record. |
| 25 | 8 | Last record offset | `u64` | Last record. |
| 33 | 7 | Reserved | `[u8; 7]` | Future header fields. |

> Empty collections have a zero count and zero record offsets. A single-record collection has matching first and last offsets.

### Root Collection

The root collection catalogs [managed collections](#managed-collections). Its header starts at file offset `80`, immediately after the file header, and has collection type `0`. Each root record stores a managed collection header as its payload.

### Managed Collections

Managed collections hold application data such as accounts and transactions. The header's `CollectionType` identifies the kind of records the collection holds and determines how to decode their payloads. Its first and last record offsets point to the managed collection's own records.

The document uses the root catalog to locate managed collection headers. At the document level, `doc.transactions` resolves the transactions collection by `CollectionType` through the root catalog.

## Records

A 48-byte record header contains the record size, links, record ID, and timestamps. Records form a doubly linked list.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | Size | `u64` | Total record size in bytes, including the header and payload. |
| 8 | 8 | Previous record offset | `u64` | Previous record. |
| 16 | 8 | Next record offset | `u64` | Next record. |
| 24 | 8 | ID | `u64` | Unique within the collection. |
| 32 | 8 | Created at | `i64` | Timestamp when the record was created. |
| 40 | 8 | Last modified at | `i64` | Timestamp when the record was last modified. |
| 48 | Variable | Payload | Varies | See layouts below. |

> Payload offset `0` is record offset `48`.

<a id="root-collection-payload"></a>

<details>
<summary><strong>Root Collection</strong></summary>
<br>

Each record in the [root collection](#root-collection) has a 40-byte payload using the [collection header layout](#collections). Its collection type identifies the managed collection.

> The full root record is 88 bytes.

</details>

<a id="account-payload"></a>

<details>
<summary><strong>Account</strong></summary>

<br>

A place where money is held, such as a bank account, wallet, or piggy bank.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 4 + N | Display name | [Length-prefixed string](#length-prefixed-string) | Account name shown in the UI. |

> The payload is `4 + N` bytes. The full record is `52 + N` bytes.

</details>

<a id="transaction-payload"></a>

<details>
<summary><strong>Transaction</strong></summary>

<br>

Represents the movement of money.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | Account ID | `u64` | Account this transaction belongs to. |
| 8 | 8 | Occurred at | `i64` | Timestamp when the transaction happened. |
| 16 | 8 | Amount | `i64` | Signed amount in centavos. |
| 24 | 4 + N | Description | [Length-prefixed string](#length-prefixed-string) | Transaction description. |

> The payload is `28 + N` bytes. The full record is `76 + N` bytes.

</details>

## Structures

<a id="length-prefixed-string"></a>

<details>
<summary><strong>Length-Prefixed String</strong></summary>
<br>

A little-endian `u32` byte length followed by UTF-8 data, with no padding or terminator. `N` is the UTF-8 byte length, not the character count.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 4 | Length | `u32` | UTF-8 byte length. |
| 4 | N | Data | UTF-8 bytes | String contents. |

Size: `4 + N` bytes. Maximum data length: `u32::MAX` bytes.

</details>

---

> **Planned:** a free-space collection will track deleted regions for reuse or compaction.

## Devlogs

See [Findings](findings.md#tracked-findings) for save correctness and retry status.

<details>
<summary><strong>Nov. 22, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 11:00 PM +08:00 | Traced the stored header position through vault creation and lazy loading. New root and missing managed collections can reach save with position zero. Proposed writing the root header before appending records and resolving managed header positions from persisted catalog entries. |
| 10:40 PM +08:00 | Verified the root catalog in a local sample using 40-byte record headers without a size field. The embedded headers stayed empty after saving, and a detached transaction header occupied the account record header. |
| 10:37 PM +08:00 | Implemented the managed collection payload writer, including the seven reserved header bytes. The first append briefly reserved header space before the original end-of-file append offset was restored. |
| 10:34 PM +08:00 | Routed account creation and deletion and transaction creation through the vault's typed collection accessors. The accounts page logs collection load errors. |
| 10:28 PM +08:00 | Added separate lazy account and transaction caches to the root collection. Both use a shared typed loader that builds indexes after registering collection-specific keys. The accessors return mutable references so the vault can write the cached collections. |
| 12:06 PM +08:00 | Added a total-size field at the start of each record, extending the record header to 48 bytes. |
| 04:06 PM +08:00 | Proposed keeping typed account and transaction collections in the vault while the root catalog handles on-disk discovery. If the vault needs an in-memory list of managed collections, use an enum that wraps each supported `Collection<T>` and expose typed accessors. |
| 11:51 AM +08:00 | Defined the root record payload as the 40-byte managed collection header, stored directly in the record. |
| 11:39 AM +08:00 | Specified the shared 40-byte collection header and removed the next-collection offset from the layout. Clarified that root records associate names with managed collection header offsets and that managed collections use their type to identify the record payload. The root catalog record encoding remains open. |
| 11:32 AM +08:00 | Specified an 80-byte file header with a fixed 68-byte display-name field and six reserved bytes. The root collection starts immediately after it. |
| 11:14 AM +08:00 | Drafted a fixed root catalog of named collections and movable typed collections. Reserved type value `0` for `Root` in the planned `CollectionType` enum and chose type-based lookup for `doc.transactions`. The root record encoding remains open. |
| 10:45 AM +08:00 | Moved pending-state cleanup and the cached count update after successful collection header writes. A retry after partial file changes remains an open issue. |
| 10:35 AM +08:00 | Corrected cancellation of pending creates and updated collection endpoints during unlinking. The follow-up changed append to check the post-delete count and refreshed the in-memory count for subsequent successful saves. |
| 10:32 AM +08:00 | Added basic deletion. The collection removes the directory entry immediately and save patches the neighboring links for persisted records. The removed record's bytes stay in the file, and new records still append at EOF. Space reclamation remains future work. |
| 09:01 AM +08:00 | Chose to remove entries from the sorted directory vector and accept the shift cost. Deferred tombstones and deep-page optimization. |
| 08:56 AM +08:00 | Explored sorted tombstone positions to translate a live-record skip into a raw offset. Indexed queries would need positions within each bucket. Maintaining the sorted positions could still shift entries in the smaller vector. |
| 08:01 AM +08:00 | Considered tombstones to avoid shifting the directory. Full scans could filter them in either direction, and indexed lookups could return `None` before pagination. Moving only `take` after lookup can fill a page while still repeating records across pages because `skip` counts stale IDs. Filtering first preserves page boundaries but costs a lookup per skipped candidate. Clean index buckets would allow direct slicing, and an ID cursor could support sequential pages. |
| 07:56 AM +08:00 | Proposed removing the directory entry and storing the disk offset in the delete delta. For an unsaved record, cancel the pending create and remove the tracked payload. |

</details>

<details>
<summary><strong>Nov. 21, 2026</strong></summary>
<br>

| Time | Logs |
| --- | --- |
| 12:06 AM +08:00 | Flagged that pending deletions must be excluded before pagination. In a directory backed by an ordered vector and an ID-to-position map, removal would require updating later positions or retaining tombstones and filtering them before pagination. |

</details>
