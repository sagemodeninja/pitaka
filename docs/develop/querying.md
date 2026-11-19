[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> Querying

# Querying

## Collection indexes

Each collection can register named equality indexes with `Collection::index(name, evaluator)`. The evaluator extracts an `IndexValue` from a record payload. Currently the only supported value is `IndexValue::U16`, and a `u16` can be converted with `.into()`.

When a document is opened, `Collection::read` builds the indexes in memory. The builder follows the collection's on-disk record links from first to last, reads each payload, and runs the registered evaluators. Each named index is a `HashMap<IndexValue, Vec<u64>>`. Each vector holds absolute file offsets of record headers in collection order. The [file format](file_layout.md) stores the records and their links. Indexes are rebuilt from those records when the document is reopened.

The document registers one index, `acct_id`, on the transaction collection. Its evaluator uses `Transaction::account_id`, so a lookup by account ID returns offsets of transactions belonging to that account. The relationship is represented by the account ID stored in the transaction payload. Looking it up requires only the transaction index.

## Querying by index

`Collection::query` builds a plan and defers payload reads until `collect()`. For example, inside the document implementation, this retrieves the second page of transactions for an account in reverse collection order:

```rust
self.transactions
    .query(&mut self.file)
    .filter_indexed("acct_id", id.into())
    .reverse()
    .skip(20)
    .take(20)
    .collect()
```

`PitakaDocument::get_acct_trans(id)` uses the same filter and reverse order and returns all matching transactions. The default order is first to last in the collection. The query reverses the matching offsets if requested, applies `skip` and `take`, then reads only the selected payloads. The defaults are zero skipped records and no limit.

Use one indexed filter per query. If a query has no filter, the index is unavailable, or the selected value has no matches, it returns an empty vector. Use `Collection::get_all` for a full collection read. Adding a second filter with the same name replaces the first value. With multiple names, the collector uses the first available index it encounters in a hash map, whose iteration order is unspecified.

The index reflects the persisted records scanned at open time. Pending appends are excluded, and saving leaves the in-memory index as it was. Reopen the document after saving to include new transactions in indexed results. A document returned by `PitakaDocument::create` gets its transaction index on the first reopen.

## Planned: Multiple filters

AND queries could intersect candidate offsets or use one index to find candidates and check the remaining conditions against their payloads. OR queries could union candidate offsets and deduplicate them. Both would determine result order before applying pagination.

## Development notes

<a id="11-17-2026"></a>
<details>
<summary><strong>Nov. 17, 2026</strong></summary>
<br>

| Time | Logs |
| -- | -- |
|11:00AM+08:00|Implemented the query builder. It makes querying "lazily" evaluate things like `skip`, `take`. It works by storing a plan that get's consumed when `collect` is called.<br><br>Next thing I am planning to make is the `reverse`. For the `get_all` query, this can be a simple start at last offset and traverse via prev. offset. For `get_by_offset`, I can probably reverse the iterator. But that's additional work? or since it's an iterator it doesn't matter?|
|11:59AM+08:00|I need a way to make `for_append` participate on the query. Right now, it appends itself on every result inc. in the `by_offset` query. It doesn't also obey the `reverse` order. If we make the `index` also contains pending items, we'll need some sort of marker on whether the record is on disk of memory.<br><br>On the same token, edited records would also need to participate, that is load the edited records instead of the one on the disk.|

</details>

<a id="11-16-2026"></a>
<details>
<summary><strong>Nov. 16, 2026</strong></summary>
<br>

Currently, the app loads **all** records for a collection. Which has some issues:

1. As the records grow, it will cause peformance issues.
2. There's no relationship whatsoever. For example, I should only be getting transactions for a specific account.

For performance, paging might help. I am thinking of building a query builder for collections. I can prob. say: `take(n).skip(n)`. This will skip `n` records and take `n` records without reading payloads. I can just read the next record offset using fixed offsets then jump. Of course, if the record is big enough it will still take some time.

Relationship will probably work in a similar manner. That is look at a record, check the payload for the relationship part, then if related, inc. it for the entire read otherwise skip.

I am also thinking of some sort of indexing. Probably store ranges of address `from` and `to` which says: "If you traverse here, you'd get records related to this FK or something. If my records or FKs will stay near each other this is probably an efficient way of doing this.

9:31 PM will do a research on indexing.
</details>
