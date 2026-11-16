pub trait StringExtensions {
    fn len_u32(&self) -> u32;
}

impl StringExtensions for String {
    fn len_u32(&self) -> u32 {
        self.len() as u32
    }
}
