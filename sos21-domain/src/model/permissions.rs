bitflags::bitflags! {
    #[derive(Default)]
    pub struct Permissions: u32 {
        const READ_ALL_USERS                             = 0b00000000000000000000000000000001;
        const READ_ALL_PROJECTS                          = 0b00000000000000000000000000000010;
        const UPDATE_ALL_USERS                           = 0b00000000000000000000000000000100;
        const UPDATE_ALL_PROJECTS                        = 0b00000000000000000000000000001000;
        const READ_ALL_FORMS                             = 0b00000000000000000000000000010000;
        const CREATE_FORMS                               = 0b00000000000000000000000000100000;
        const READ_ALL_FORM_ANSWERS                      = 0b00000000000000000000000001000000;
        const CREATE_FILES                               = 0b00000000000000000000000010000000;
        const SHARE_FILES                                = 0b00000000000000000000000100000000;
        const DISTRIBUTE_FILES                           = 0b00000000000000000000001000000000;
        const READ_ALL_FILE_DISTRIBUTIONS                = 0b00000000000000000000010000000000;
        const READ_ALL_REGISTRATION_FORMS                = 0b00000000000000000000100000000000;
        const READ_ALL_REGISTRATION_FORM_ANSWERS         = 0b00000000000000000001000000000000;
        const CREATE_REGISTRATION_FORMS                  = 0b00000000000000000010000000000000;
        const ANSWER_REGISTRATION_FORMS                  = 0b00000000000000000100000000000000;
        const READ_ALL_USER_INVITATIONS                  = 0b00000000000000001000000000000000;
        const CREATE_USER_INVITATIONS                    = 0b00000000000000010000000000000000;
        const UPDATE_ALL_FORMS                           = 0b00000000000000100000000000000000;
        const UPDATE_NOT_STARTED_OWNING_FORMS            = 0b00000000000001000000000000000000;
        const UPDATE_ALL_FORM_ANSWERS                    = 0b00000000000010000000000000000000;
        const UPDATE_FORM_ANSWERS_IN_PERIOD              = 0b00000000000100000000000000000000;
        const UPDATE_MEMBER_PROJECTS_IN_PERIOD           = 0b00000000001000000000000000000000;
        const UPDATE_ALL_PENDING_PROJECTS                = 0b00000000010000000000000000000000;
        const UPDATE_OWNING_PENDING_PROJECTS_IN_PERIOD   = 0b00000000100000000000000000000000;
        const UPDATE_ALL_REGISTRATION_FORM_ANSWERS       = 0b00000001000000000000000000000000;
        const UPDATE_REGISTRATION_FORM_ANSWERS_IN_PERIOD = 0b00000010000000000000000000000000;
        const UPDATE_PROJECT_CATEGORY                    = 0b00000100000000000000000000000000;
        const UPDATE_PENDING_PROJECT_CATEGORY            = 0b00001000000000000000000000000000;
    }
}
