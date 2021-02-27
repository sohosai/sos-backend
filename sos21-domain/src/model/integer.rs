use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use thiserror::Error;

use crate::model::bound::{Bound, Bounded};

/// A range-limited integer.
///
/// This provides a wrapper to validate that the value of integer is
/// between `Lower` and `Upper` bounds.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedInteger<Lower, Upper, T> {
    _lower: PhantomData<Lower>,
    _upper: PhantomData<Upper>,
    inner: T,
}

pub type BoundedInteger<Min, Max, T> = LimitedInteger<Bounded<Min>, Bounded<Max>, T>;

impl<Lower, Upper, T> LimitedInteger<Lower, Upper, T> {
    pub fn new(i: T) -> Result<Self, BoundError>
    where
        Lower: Bound<T>,
        Upper: Bound<T>,
        T: PartialOrd,
    {
        if let Some(lower) = Lower::limit() {
            if i < lower {
                return Err(BoundError { _priv: () });
            }
        }
        if let Some(upper) = Upper::limit() {
            if i > upper {
                return Err(BoundError { _priv: () });
            }
        }

        Ok(LimitedInteger {
            _upper: PhantomData,
            _lower: PhantomData,
            inner: i,
        })
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[derive(Debug, Error, Clone)]
#[error("the value of integer is out of bounds")]
pub struct BoundError {
    _priv: (),
}

impl<Lower, Upper, T: Display> Display for LimitedInteger<Lower, Upper, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as Display>::fmt(&self.inner, f)
    }
}

impl<Lower, Upper, T: Debug> Debug for LimitedInteger<Lower, Upper, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as Debug>::fmt(&self.inner, f)
    }
}

impl<Lower, Upper, T> Serialize for LimitedInteger<Lower, Upper, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, Lower, Upper, T> Deserialize<'de> for LimitedInteger<Lower, Upper, T>
where
    Lower: Bound<T>,
    Upper: Bound<T>,
    T: Deserialize<'de> + PartialOrd,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        LimitedInteger::new(T::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::LimitedInteger;
    use crate::model::bound::{Bounded, Unbounded};

    #[test]
    fn test_bounded() {
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::P3>, i64>::new(0).is_ok());
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::P3>, i64>::new(-1).is_ok());
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::P3>, i64>::new(3).is_ok());
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::P3>, i64>::new(4).is_err());
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::P2>, i64>::new(3).is_err());
        assert!(LimitedInteger::<Unbounded, Bounded<typenum::U2>, u64>::new(2).is_ok());
        assert!(LimitedInteger::<Bounded<typenum::P1>, Bounded<typenum::P3>, i64>::new(1).is_ok());
        assert!(LimitedInteger::<Bounded<typenum::P1>, Bounded<typenum::P3>, i64>::new(0).is_err());
        assert!(
            LimitedInteger::<Bounded<typenum::Z0>, Bounded<typenum::P3>, i64>::new(-1).is_err()
        );
        assert!(LimitedInteger::<Bounded<typenum::N2>, Bounded<typenum::P3>, i64>::new(-1).is_ok());
    }
}
