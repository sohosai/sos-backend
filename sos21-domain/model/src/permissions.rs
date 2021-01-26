bitflags::bitflags! {
    #[derive(Default)]
    pub struct Permissions: u32 {
        const READ_ALL_USERS    = 0b00000001;
        const READ_ALL_PROJECTS = 0b00000010;
    }
}
