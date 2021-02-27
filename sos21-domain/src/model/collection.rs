use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::marker::PhantomData;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use thiserror::Error;

use crate::model::bound::{Bound, Bounded};

macro_rules! length_limited_collection {
    ($name:ident, $t:ident) => {
        #[derive(Clone)]
        pub struct $name<Lower, Upper, T> {
            _lower: PhantomData<Lower>,
            _upper: PhantomData<Upper>,
            inner: $t<T>,
        }

        #[allow(dead_code)]
        impl<Lower, Upper, T> $name<Lower, Upper, T> {
            pub fn new(v: $t<T>) -> Result<Self, LengthError<Lower, Upper>>
            where
                Lower: Bound<usize>,
                Upper: Bound<usize>,
            {
                let len = v.len();
                if let Some(lower) = Lower::limit() {
                    if len < lower {
                        return Err(LengthError {
                            kind: LengthErrorKind::TooShort,
                            _upper: PhantomData,
                            _lower: PhantomData,
                        });
                    }
                }
                if let Some(upper) = Upper::limit() {
                    if len > upper {
                        return Err(LengthError {
                            kind: LengthErrorKind::TooLong,
                            _upper: PhantomData,
                            _lower: PhantomData,
                        });
                    }
                }

                Ok($name {
                    _upper: PhantomData,
                    _lower: PhantomData,
                    inner: v,
                })
            }

            pub fn len(&self) -> usize {
                self.inner.len()
            }

            pub fn iter(&self) -> impl Iterator<Item = &T> {
                self.inner.iter()
            }

            pub fn as_inner(&self) -> &$t<T> {
                &self.inner
            }

            pub fn into_inner(self) -> $t<T> {
                self.inner
            }
        }

        impl<Lower, Upper, T: Debug> Debug for $name<Lower, Upper, T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                <$t<T> as Debug>::fmt(&self.inner, f)
            }
        }

        impl<Lower, Upper, T> Serialize for $name<Lower, Upper, T>
        where
            $t<T>: Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.inner.serialize(serializer)
            }
        }

        impl<'de, Lower, Upper, T> Deserialize<'de> for $name<Lower, Upper, T>
        where
            Lower: Bound<usize>,
            Upper: Bound<usize>,
            $t<T>: Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                $name::new($t::<T>::deserialize(deserializer)?).map_err(de::Error::custom)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    TooLong,
    TooShort,
}

pub type BoundedLengthError<Min, Max> = LengthError<Bounded<Min>, Bounded<Max>>;

#[derive(Error, Clone)]
#[error("the collection's length is out of bounds")]
pub struct LengthError<Lower, Upper> {
    kind: LengthErrorKind,
    _lower: PhantomData<Lower>,
    _upper: PhantomData<Upper>,
}

impl<Lower, Upper> Debug for LengthError<Lower, Upper> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LengthError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl<Lower, Upper> LengthError<Lower, Upper> {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }
}

length_limited_collection! { LengthLimitedVec, Vec }
length_limited_collection! { LengthLimitedSet, HashSet }

impl<Lower, Upper, T: PartialEq> PartialEq for LengthLimitedVec<Lower, Upper, T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<Lower, Upper, T: Eq> Eq for LengthLimitedVec<Lower, Upper, T> {}

impl<Lower, Upper, T: Eq + Hash> PartialEq for LengthLimitedSet<Lower, Upper, T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<Lower, Upper, T: Eq + Hash> Eq for LengthLimitedSet<Lower, Upper, T> {}

impl<Lower, Upper, T> LengthLimitedSet<Lower, Upper, T>
where
    Upper: Bound<usize>,
    T: Eq + Hash,
{
    pub fn insert(&mut self, value: T) -> Result<bool, LengthError<Lower, Upper>> {
        let is_inserted = self.inner.insert(value);
        if let Some(upper) = Upper::limit() {
            if self.inner.len() > upper {
                return Err(LengthError {
                    kind: LengthErrorKind::TooLong,
                    _upper: PhantomData,
                    _lower: PhantomData,
                });
            }
        }
        Ok(is_inserted)
    }

    pub fn difference<'a>(
        &'a self,
        other: &'a LengthLimitedSet<Lower, Upper, T>,
    ) -> impl Iterator<Item = &'a T> + 'a {
        self.inner.difference(&other.inner)
    }

    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        Q: Hash + Eq,
        T: Borrow<Q>,
    {
        self.inner.contains(value)
    }
}

pub type LengthBoundedVec<Min, Max, T> = LengthLimitedVec<Bounded<Min>, Bounded<Max>, T>;
// pub type LengthBoundedSet<Min, Max, T> = LengthLimitedSet<Bounded<Min>, Bounded<Max>, T>;

#[cfg(test)]
mod tests {
    use super::{LengthLimitedSet, LengthLimitedVec};
    use crate::model::bound::{Bounded, Unbounded};

    #[test]
    fn test_bounded_vec() {
        assert!(LengthLimitedVec::<Unbounded, Bounded<typenum::U3>, i32>::new(vec![]).is_ok());
        assert!(
            LengthLimitedVec::<Unbounded, Bounded<typenum::U3>, i32>::new(vec![1, 2, 3]).is_ok()
        );
        assert!(
            LengthLimitedVec::<Unbounded, Bounded<typenum::U3>, i32>::new(vec![1, 2, 3, 4])
                .is_err()
        );
        assert!(
            LengthLimitedVec::<Unbounded, Bounded<typenum::U2>, i32>::new(vec![1, 2, 3]).is_err()
        );
        assert!(
            LengthLimitedVec::<Bounded<typenum::U1>, Bounded<typenum::U3>, i32>::new(vec![0])
                .is_ok()
        );
        assert!(
            LengthLimitedVec::<Bounded<typenum::U1>, Bounded<typenum::U3>, i32>::new(vec![])
                .is_err()
        );
        assert!(
            LengthLimitedVec::<Bounded<typenum::U2>, Bounded<typenum::U3>, String>::new(vec![
                "a".to_string()
            ])
            .is_err()
        );
    }

    #[test]
    fn test_bounded_set() {
        assert!(
            LengthLimitedSet::<Unbounded, Bounded<typenum::U3>, i32>::new(maplit::hashset![])
                .is_ok()
        );
        assert!(
            LengthLimitedSet::<Unbounded, Bounded<typenum::U3>, i32>::new(maplit::hashset![
                1, 2, 3
            ])
            .is_ok()
        );
        assert!(
            LengthLimitedSet::<Unbounded, Bounded<typenum::U3>, i32>::new(maplit::hashset![
                1, 2, 3, 4
            ])
            .is_err()
        );
        assert!(
            LengthLimitedSet::<Unbounded, Bounded<typenum::U3>, i32>::new(maplit::hashset![
                1, 2, 3, 3
            ])
            .is_ok()
        );
        assert!(
            LengthLimitedSet::<Unbounded, Bounded<typenum::U2>, i32>::new(maplit::hashset![
                1, 2, 3
            ])
            .is_err()
        );
        assert!(
            LengthLimitedSet::<Bounded<typenum::U1>, Bounded<typenum::U3>, i32>::new(
                maplit::hashset![1]
            )
            .is_ok()
        );
        assert!(
            LengthLimitedSet::<Bounded<typenum::U1>, Bounded<typenum::U3>, i32>::new(
                maplit::hashset![]
            )
            .is_err()
        );
        assert!(
            LengthLimitedSet::<Bounded<typenum::U2>, Bounded<typenum::U3>, String>::new(
                maplit::hashset!["a".to_string()]
            )
            .is_err()
        );
    }
}
