/// Defines a `bounds` method returning the first and last
/// item from an Iterator.
pub trait Bounds: Iterator
where
    Self::Item: Copy,
{
    /// Get the first and last items from an iterator.
    ///
    /// If the iterator has only one item, that item will
    /// be returned as both the lower and upper bound.
    fn bounds(&mut self) -> Option<(Self::Item, Self::Item)> {
        let min = self.next();
        min.map(move |min| (min, self.last().unwrap_or(min)))
    }
}

impl<T: Copy, I: Iterator<Item = T>> Bounds for I {}
