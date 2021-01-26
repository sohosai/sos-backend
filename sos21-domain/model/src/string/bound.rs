use typenum::Unsigned;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounded<N>(N);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unbounded;

pub trait Bound {
    fn limit() -> Option<usize>;
}

impl Bound for Unbounded {
    fn limit() -> Option<usize> {
        None
    }
}

impl<N: Unsigned> Bound for Bounded<N> {
    fn limit() -> Option<usize> {
        Some(N::to_usize())
    }
}
