use crate::model::user::UserEmailAddress;

pub trait ConfigContext {
    fn administrator_email(&self) -> &UserEmailAddress;
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
        }
    }
}

impl<C: ConfigContext> ConfigContext for &C {
    fn administrator_email(&self) -> &UserEmailAddress {
        <C as ConfigContext>::administrator_email(self)
    }
}
