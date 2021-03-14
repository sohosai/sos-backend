bitflags::bitflags! {
    #[derive(Default)]
    pub struct Permissions: u32 {
        const READ_ALL_USERS        = 0b00000001;
        const READ_ALL_PROJECTS     = 0b00000010;
        const UPDATE_ALL_USERS      = 0b00000100;
        const UPDATE_ALL_PROJECTS   = 0b00001000;
        const READ_ALL_FORMS        = 0b00010000;
        const CREATE_FORMS          = 0b00100000;
        const READ_ALL_FORM_ANSWERS = 0b01000000;
        const CREATE_FILES          = 0b10000000;
    }
}
