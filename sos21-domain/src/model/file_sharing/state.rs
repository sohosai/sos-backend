#[derive(Debug, Clone, Copy)]
pub enum FileSharingState {
    Active,
    Revoked,
    Expired,
}

impl FileSharingState {
    pub fn is_active(&self) -> bool {
        matches!(self, FileSharingState::Active)
    }
}
