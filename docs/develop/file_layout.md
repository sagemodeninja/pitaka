[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> File Layout

# File Layout

> **Warning:** This format is unstable. Any update may make existing saves unreadable. No migration or recovery tools are provided, and compatibility fixes are not planned during development.

Sizes and offsets are in bytes. Integers are little-endian. Table offsets are relative to the structure shown; stored offsets are absolute `u64` file positions, with `0` meaning no target. Reserved bytes must be zero. Types describe field encodings; sizes describe serialized bytes, not Rust memory layouts.

## Contents

- [File Header](#file-header)
- [Collections](#collections)
- [Records](#records)
  - [Account Payload](#account-payload)
  - [Transaction Payload](#transaction-payload)
- [Structures](#structures)
  - [Length-Prefixed String](#length-prefixed-string)

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
| 8 | 1 | Collection type | `u8` | Record type identifier. |
| 9 | 8 | Next record ID | `u64` | Next record ID; never reduced when records are deleted. |
| 17 | 8 | Record count | `u64` | Persisted record count. |
| 25 | 8 | First record offset | `u64` | First record. |
| 33 | 8 | Last record offset | `u64` | Last record. |
| 41 | 7 | Reserved | `[u8; 7]` | Future header fields. |

> Empty collections have a zero count and zero record offsets. A single-record collection has matching first and last offsets.

## Records

A 16-byte link header followed by a collection-specific payload. Records form a doubly linked list.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | Previous record offset | `u64` | Previous record. |
| 8 | 8 | Next record offset | `u64` | Next record. |
| 16 | Variable | Payload | Varies | See layouts below. |

> Payload offsets start after the link header.

<a id="account-payload"></a>

<details>
<summary><strong>Account Payload</strong></summary>

<br>

A place where money is held, such as a bank account, wallet, or piggy bank.

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 2 | ID | `u16` | Unique within the account collection. |
| 2 | 4 + N | Display name | [Length-prefixed string](#length-prefixed-string) | Account name shown in the UI. |

Size: `6 + N` bytes; `22 + N` including the link header.

</details>

<a id="transaction-payload"></a>

<details>
<summary><strong>Transaction</strong></summary>

<br>

| Offset | Size | Field | Type | Description |
| --- | --- | --- | --- | --- |
| 0 | 8 | ID | `u64` | Unique within the transaction collection. |
| 8 | 2 | Account ID | `u16` | Account this transaction belongs to. |
| 10 | 4 + N | Description | [Length-prefixed string](#length-prefixed-string) | Transaction description. |
| 14 + N | 8 | Amount | `i64` | Signed amount in centavos. |

Size: `22 + N` bytes; `38 + N` including the link header.

**Planned:** a free-space collection will track deleted regions for reuse or compaction.

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
