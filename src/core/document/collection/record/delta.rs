pub enum RecordDelta {
    Create(u64),
    Update(u64),
    Delete(u64),
}
