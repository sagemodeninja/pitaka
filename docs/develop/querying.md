[Pitaka](../../README.md) <span style="color: #888;">/</span> [Development](README.md) <span style="color: #888;">/</span> File Layout

> This will serve as both official documentation and devlogs for my thoughts on how to implement the querying system.

# Querying

## Relationships

For records with relationships such as `Accounts` and `Transactions`, an "index" is constructed on read. This index is a simple hash map of the parent ID and child IDs (`HashMap<u16, Vec<u64>`). The "indexer" simply traverses the records, and populates child IDs with the parent ID as key.

## Notes

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