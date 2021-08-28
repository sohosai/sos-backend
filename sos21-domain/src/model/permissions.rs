use std::ops::BitOr;

use enumflags2::{bitflags, make_bitflags, BitFlags};
use paste::paste;

macro_rules! define_permissions {
    (#[$attr:meta] $vis:vis enum $ty_name:ident: $repr:ident {
        $($flag_vis:vis $flag_name:ident),* $(,)?
    }) => {
        paste! {
            #[bitflags]
            #[repr($repr)]
            #[$attr]
            enum [<$ty_name Flags>] {
                $([<$flag_name:lower:camel>]),*
            }

            #[$attr]
            pub struct $ty_name(BitFlags<[<$ty_name Flags>]>);

            impl $ty_name {
                $(
                    $flag_vis const $flag_name: $ty_name =
                        $ty_name(make_bitflags!([<$ty_name Flags>]::{[<$flag_name:lower:camel>]}));
                )*
            }
        }
    }
}

define_permissions! {
    #[derive(Copy, Clone, Debug)]
    pub enum Permissions: u32 {
        pub READ_ALL_USERS,
        pub READ_ALL_PROJECTS,
        pub UPDATE_ALL_USERS,
        pub UPDATE_ALL_PROJECTS,
        pub READ_ALL_FORMS,
        pub CREATE_FORMS,
        pub READ_ALL_FORM_ANSWERS,
        pub CREATE_FILES,
        pub SHARE_FILES,
        pub DISTRIBUTE_FILES,
        pub READ_ALL_FILE_DISTRIBUTIONS,
        pub READ_ALL_REGISTRATION_FORMS,
        pub READ_ALL_REGISTRATION_FORM_ANSWERS,
        pub CREATE_REGISTRATION_FORMS,
        pub ANSWER_REGISTRATION_FORMS,
        pub READ_ALL_USER_INVITATIONS,
        pub CREATE_USER_INVITATIONS,
        pub UPDATE_ALL_FORMS,
        pub UPDATE_NOT_STARTED_OWNING_FORMS,
        pub UPDATE_ALL_FORM_ANSWERS,
        pub UPDATE_FORM_ANSWERS_IN_PERIOD,
        pub UPDATE_MEMBER_PROJECTS_IN_PERIOD,
        pub UPDATE_ALL_PENDING_PROJECTS,
        pub UPDATE_OWNING_PENDING_PROJECTS_IN_PERIOD,
        pub UPDATE_ALL_REGISTRATION_FORM_ANSWERS,
        pub UPDATE_REGISTRATION_FORM_ANSWERS_IN_PERIOD,
        pub UPDATE_PROJECT_CATEGORY,
        pub UPDATE_PENDING_PROJECT_CATEGORY,
    }
}

impl Permissions {
    pub fn all() -> Self {
        Permissions(BitFlags::all())
    }

    pub fn contains(&self, other: Permissions) -> bool {
        self.0.contains(other.0)
    }

    pub fn union(self, other: Permissions) -> Self {
        Permissions(self.0 | other.0)
    }
}

impl BitOr for Permissions {
    type Output = Permissions;
    fn bitor(self, other: Permissions) -> Permissions {
        self.union(other)
    }
}
