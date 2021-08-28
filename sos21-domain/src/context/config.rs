use crate::model::project::ProjectCategory;
use crate::model::project_creation_period::ProjectCreationPeriod;
use crate::model::user::UserEmailAddress;

pub trait ConfigContext {
    fn administrator_email(&self) -> &UserEmailAddress;
    fn project_creation_period_for(&self, category: ProjectCategory) -> ProjectCreationPeriod;
}

#[macro_export]
macro_rules! delegate_config_context {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? ConfigContext for $ty:ty {
        $sel:ident $target:block
    }) => {
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::ConfigContext for $ty {
            fn administrator_email(&$sel) -> &$crate::model::user::UserEmailAddress {
                $target.administrator_email()
            }
            fn project_creation_period_for(
                &$sel,
                category: $crate::model::project::ProjectCategory,
            ) -> $crate::model::project_creation_period::ProjectCreationPeriod {
                $target.project_creation_period_for(category)
            }
        }
    }
}

impl<C: ConfigContext> ConfigContext for &C {
    fn administrator_email(&self) -> &UserEmailAddress {
        <C as ConfigContext>::administrator_email(self)
    }

    fn project_creation_period_for(&self, category: ProjectCategory) -> ProjectCreationPeriod {
        <C as ConfigContext>::project_creation_period_for(self, category)
    }
}
