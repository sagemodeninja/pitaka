pub enum RecordDelta {
    Create(u64),
    Update(u64),

    /// Deletes the record at the given file offset.
    Delete(u64),
}
