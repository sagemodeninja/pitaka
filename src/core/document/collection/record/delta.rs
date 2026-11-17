pub struct RecordDelta {
    pub record_id: u64,
    pub delta_type: RecordDeltaType,
}

pub enum RecordDeltaType {
    Create,
    Update,
    Delete,
}

impl RecordDelta {
    pub fn create(id: u64) -> Self {
        Self {
            record_id: id,
            delta_type: RecordDeltaType::Create,
        }
    }

    pub fn update(id: u64) -> Self {
        Self {
            record_id: id,
            delta_type: RecordDeltaType::Update,
        }
    }

    pub fn delete(id: u64) -> Self {
        Self {
            record_id: id,
            delta_type: RecordDeltaType::Delete,
        }
    }
}
