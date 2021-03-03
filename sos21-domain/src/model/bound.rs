use typenum::{Integer, Unsigned};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounded<N>(N);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unbounded;

pub trait Bound<T> {
    fn limit() -> Option<T>;
}

impl<T> Bound<T> for Unbounded {
    fn limit() -> Option<T> {
        None
    }
}

impl<N: Unsigned> Bound<usize> for Bounded<N> {
    fn limit() -> Option<usize> {
        Some(N::to_usize())
    }
}

impl<N: Unsigned> Bound<u64> for Bounded<N> {
    fn limit() -> Option<u64> {
        Some(N::to_u64())
    }
}

impl<N: Unsigned> Bound<u16> for Bounded<N> {
    fn limit() -> Option<u16> {
        Some(N::to_u16())
    }
}

impl<N: Integer> Bound<i64> for Bounded<N> {
    fn limit() -> Option<i64> {
        Some(N::to_i64())
    }
}
