#[derive(Debug, Clone, Copy)]
pub struct UserFileUsage(u64);

impl UserFileUsage {
    pub fn from_number_of_bytes(usage: u64) -> Self {
        UserFileUsage(usage)
    }

    pub fn to_number_of_bytes(&self) -> u64 {
        self.0
    }
}
