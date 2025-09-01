pub trait SaturatingExt: Iterator {
    fn saturating(self) -> Saturating<Self>
    where
        Self: Sized,
    {
        Saturating::new(self)
    }
}

impl<I> SaturatingExt for I where I: Iterator + ?Sized {}

#[derive(Clone, Debug)]
pub struct Saturating<I>
where
    I: Iterator,
{
    inner: I,
    last: Option<I::Item>,
}

impl<I> Saturating<I>
where
    I: Iterator,
{
    pub fn new(inner: I) -> Self {
        Self { inner, last: None }
    }
}

impl<I> Iterator for Saturating<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if let Some(item) = item {
            self.last = Some(item.clone());
            Some(item)
        } else {
            self.last.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_saturates() {
        let mut iter = (0..3).saturating();

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(2));
    }
}
