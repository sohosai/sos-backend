#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileSize(u64);

impl FileSize {
    pub fn from_number_of_bytes(size: u64) -> Self {
        FileSize(size)
    }

    pub fn to_number_of_bytes(&self) -> u64 {
        self.0
    }
}
