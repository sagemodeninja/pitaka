use crate::core::vault::collection::RecordPayload;

pub enum RecordDelta<T: RecordPayload> {
    // ID, Item
    Create(u64, T),

    // Offset, Id, Item
    Update(u64, u64, T),

    // Offset
    Delete(u64),
}
