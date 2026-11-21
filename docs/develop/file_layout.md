[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> File Layout

# File Layout

> **Warning:** This format is unstable. Any update may make existing saves unreadable. No migration or recovery tools are provided, and compatibility fixes are not planned at this point of the development.

## Contents

- [Format Rules](#format-rules)
- [File Header](#file-header)
- [Collections](#collections)
- [Records](#records)
  - [Account Payload](#account-payload)
  - [Transaction Payload](#transaction-payload)
- [Structures](#structures)
  - [Length-Prefixed String](#length-prefixed-string)

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

16 bytes at the start of the file.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 4 | Magic bytes | `[u8; 4]` | `b"PTKA"`. |
| 4 | 2 | Format version | `u16` | Reject unsupported versions. |
| 6 | 10 | Reserved | `[u8; 10]` | Future header fields. |

## Collections

Collections form a singly linked list of 48-byte headers, including 7 reserved bytes for future fields. The first header starts at file offset `16`.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | Next collection offset | `u64` | Next collection header. |
| 8 | 1 | Record type | `u8` | Record type identifier. |
| 9 | 8 | Next record ID | `u64` | Next record ID; never reduced when records are deleted. |
| 17 | 8 | Record count | `u64` | Persisted record count. |
| 25 | 8 | First record offset | `u64` | First record. |
| 33 | 8 | Last record offset | `u64` | Last record. |
| 41 | 7 | Reserved | `[u8; 7]` | Future header fields. |

> Empty collections have a zero count and zero record offsets. A single-record collection has matching first and last offsets.

## Records

A 40-byte record header contains the links, record ID, and timestamps. Records form a doubly linked list.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | Previous record offset | `u64` | Previous record. |
| 8 | 8 | Next record offset | `u64` | Next record. |
| 16 | 8 | ID | `u64` | Unique within the collection. |
| 24 | 8 | Created at | `i64` | Timestamp when the record was created. |
| 32 | 8 | Last modified at | `i64` | Timestamp when the record was last modified. |
| 40 | Variable | Payload | Varies | See layouts below. |

> Payload offset `0` is record offset `40`.

<a id="account-payload"></a>

<details>
<summary><strong>Account Payload</strong></summary>

<br>

A place where money is held, such as a bank account, wallet, or piggy bank.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 4 + N | Display name | [Length-prefixed string](#length-prefixed-string) | Account name shown in the UI. |

> The payload is `4 + N` bytes. The full record is `44 + N` bytes.

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

> The payload is `28 + N` bytes. The full record is `68 + N` bytes.

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
