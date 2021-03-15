use crate::model::object::{Object, ObjectId};

#[async_trait::async_trait]
pub trait ObjectRepository {
    type OutOfLimitSizeError: std::error::Error;

    /// Stores an object.
    async fn store_object(&self, object: Object) -> anyhow::Result<()>;
    /// Stores an object while specifying the limit of object size.
    ///
    /// When the size of the object is larger than the `limit` ( `size > limit` ),
    /// the object will not be stored and `Self::OutOfLimitSizeError` is returned.
    async fn store_object_with_limit(
        &self,
        object: Object,
        limit: u64,
    ) -> anyhow::Result<Result<(), Self::OutOfLimitSizeError>>;
    async fn get_object(&self, id: ObjectId) -> anyhow::Result<Option<Object>>;
}

#[macro_export]
macro_rules! delegate_object_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? ObjectRepository for $ty:ty {
        Self { $target_ty:ty },
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::ObjectRepository for $ty {
            type OutOfLimitSizeError = <$target_ty as $crate::context::ObjectRepository>::OutOfLimitSizeError;

            async fn store_object(
                &$sel,
                object: $crate::model::object::Object,
            ) -> ::anyhow::Result<()> {
                $target.store_object(object).await
            }
            async fn store_object_with_limit(
                &$sel,
                object: $crate::model::object::Object,
                limit: u64,
            ) -> ::anyhow::Result<Result<(), Self::OutOfLimitSizeError>> {
                $target.store_object_with_limit(object, limit).await
            }
            async fn get_object(
                &$sel,
                id: $crate::model::object::ObjectId
            ) -> ::anyhow::Result<Option<$crate::model::object::Object>> {
                $target.get_object(id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: ObjectRepository + Sync> ObjectRepository for &C {
    type OutOfLimitSizeError = <C as ObjectRepository>::OutOfLimitSizeError;

    async fn store_object(&self, object: Object) -> anyhow::Result<()> {
        <C as ObjectRepository>::store_object(self, object).await
    }

    async fn store_object_with_limit(
        &self,
        object: Object,
        limit: u64,
    ) -> anyhow::Result<Result<(), Self::OutOfLimitSizeError>> {
        <C as ObjectRepository>::store_object_with_limit(self, object, limit).await
    }

    async fn get_object(&self, id: ObjectId) -> anyhow::Result<Option<Object>> {
        <C as ObjectRepository>::get_object(self, id).await
    }
}
