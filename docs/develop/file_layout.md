# File Layout

All offsets and sizes are measured in bytes. Multi-byte integers use little-endian encoding. Each pointer is stored as a `u64` offset measured from the beginning of the file.

## Header

The file header occupies 16 bytes. Offsets in this table are absolute file offsets.

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 4 | Magic bytes | The exact byte sequence `b"PTKA"`, which identifies the Pitaka file format. |
| 4 | 2 | Format version | A `u16` identifying the file-format version. Readers use this value to select the appropriate layout and reject unsupported versions. |
| 6 | 10 | Reserved | Space reserved for future header fields. Writers must fill these bytes with zeros. |

## Collection Pointers

Each collection-pointer entry occupies 24 bytes and identifies a collection's type and header location. Entries form a singly linked list, allowing new entries to be appended and linked without relocating existing entries.

The offsets in this table are relative to the start of an entry. Pointer values stored within the entry are absolute file offsets.

> The first collection-pointer entry begins immediately after the file header, at absolute offset 16

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 1 | Collection type | A `u8` identifying the kind of records stored in the collection. |
| 1 | 8 | Next pointer | Offset of the next collection-pointer entry, or `0` if this is the last entry. |
| 9 | 8 | Collection position | Offset of the collection header. Must point to a valid collection header outside the file header. |
| 17 | 7 | Reserved | Space reserved for future entry fields. Writers must fill these bytes with zeros. |

## Collection Header

Each collection header occupies 32 bytes and stores the persisted record count and the locations of the first and last records. It contains no reserved bytes in the current implementation.

The offsets in this table are relative to the start of the collection header. Stored record pointers are absolute file offsets.

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 4 | Record count | A `u32` containing the number of records stored in the collection. |
| 4 | 8 | First record pointer | Offset of the first record, or `0` if the collection is empty. |
| 12 | 8 | Last record pointer | Offset of the last record, or `0` if the collection is empty. |
| 20 | 12 | Reserved | Space reserved for future entry fields. Writers must fill these bytes with zeros. |

An empty collection has a record count of `0` and both record pointers set to `0`. For a collection containing one record, the first and last record pointers refer to the same record.

## Records

Each record contains a 16-byte link header followed by its payload. Records form a doubly linked list within their collection. The payload layout depends on the collection type, so the total record size may vary.

Offsets in this table are relative to the start of the record. Stored record pointers are absolute file offsets.

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 8 | Previous record pointer | Offset of the previous record in the collection, or `0` if this is the first record. |
| 8 | 8 | Next record pointer | Offset of the next record in the collection, or `0` if this is the last record. |
| 16 | Variable | Payload | The record's data, encoded according to the collection type. |

### Transaction Payload

Offsets in this table are relative to the start of the payload, which begins 16 bytes after the start of the record. `N` is the description's UTF-8 byte length, not its character count.

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 8 | Description length | Number of UTF-8 bytes in the description. The current implementation encodes this as a little-endian `usize`, which occupies 8 bytes on the current 64-bit target. |
| 8 | N | Description | Exactly `N` UTF-8 bytes, without a null terminator or padding. |
| 8 + N | 8 | Amount | An `i64` storing the signed amount in centavos. |

The transaction payload occupies `16 + N` bytes. Including the link header, the complete transaction record occupies `32 + N` bytes.

> Portability note: the current description-length implementation depends on the platform's `usize` width. A fixed-width integer is needed to make this field consistent across 32-bit and 64-bit targets.

> Planned: a dedicated free-space collection will track the offsets and lengths of reusable regions left by deleted records. Writers can reuse these regions or compact the file when reclaimable space becomes excessive.
