//! A set of spec implementations stored for easy lookup with
//! [`Schema.activations`](Schema.html#activations).

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use crate::{Feature, Version};

/// Implementations stores a set of implementations indexed by
/// spec identity and version.
pub struct Implementations<T>(HashMap<Cow<'static, str>, BTreeMap<Version, T>>);

impl<T> Implementations<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn provide<Id, V>(mut self, identity: Id, version: V, implementation: T) -> Self
    where
        Id: Into<Cow<'static, str>>,
        V: Into<Version>,
    {
        self.0
            .entry(identity.into())
            .or_default()
            .entry(version.into())
            .or_insert(implementation);
        self
    }

    pub(crate) fn find<'a, S: AsRef<str>>(
        &'a self,
        identity: S,
        version: &'a Version,
    ) -> Find<'a, T, impl Iterator<Item = Found<'a, T>>> {
        let versions = self.0.get(identity.as_ref());
        match versions {
            Some(versions) => versions
                .range(version..&Version(version.0, u64::MAX))
                .filter(move |(impl_version, _)| impl_version.satisfies(version))
                .into(),
            None => Find::None,
        }
    }

    pub fn find_feature<'a>(
        &'a self,
        feature: &'a Feature,
    ) -> Find<'a, T, impl Iterator<Item = Found<'a, T>>> {
        self.find(&feature.spec.identity, &feature.spec.version)
    }
}

pub type Found<'a, T> = (&'a Version, &'a T);

pub enum Find<'a, T: 'a, I: Iterator<Item = Found<'a, T>>> {
    None,
    Found(I),
}

impl<'a, T, I> Iterator for Find<'a, T, I>
where
    T: 'a,
    I: Iterator<Item = Found<'a, T>>,
{
    type Item = Found<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::Found(iter) => iter.next(),
        }
    }
}

impl<'a, T, I> From<I> for Find<'a, T, I>
where
    T: 'a,
    I: Iterator<Item = Found<'a, T>>,
{
    fn from(iter: I) -> Self {
        Self::Found(iter)
    }
}

impl<T> Default for Implementations<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bounds, Implementations, Version};

    #[test]
    fn it_finds_exact_matches() {
        let identity = "https://spec.example.com/specA";
        let impls = Implementations::new()
            .provide(identity, Version(0, 9), "too small")
            .provide(identity, Version(1, 0), "Specification A")
            .provide(identity, Version(2, 0), "too big");

        assert_eq!(
            impls.find(&identity, &Version(1, 0)).collect::<Vec<_>>(),
            vec![(&Version(1, 0), &"Specification A"),]
        );

        assert_eq!(
            impls.find(&identity, &Version(1, 0)).bounds(),
            Some((
                (&Version(1, 0), &"Specification A"),
                (&Version(1, 0), &"Specification A"),
            ))
        );
    }

    #[test]
    fn it_finds_satisfying_matches() {
        let identity = "https://spec.example.com/specA";
        let impls = Implementations::new()
            .provide(identity, Version(0, 9), "too small")
            .provide(identity, Version(2, 99), "2.99")
            .provide(identity, Version(1, 0), "1.0")
            .provide(identity, Version(1, 2), "1.2")
            .provide(identity, Version(1, 3), "1.3")
            .provide(identity, Version(1, 5), "1.5")
            .provide(identity, Version(2, 0), "2.0");

        assert_eq!(
            impls.find(&identity, &Version(1, 0)).collect::<Vec<_>>(),
            vec![
                (&Version(1, 0), &"1.0"),
                (&Version(1, 2), &"1.2"),
                (&Version(1, 3), &"1.3"),
                (&Version(1, 5), &"1.5"),
            ]
        );

        assert_eq!(
            impls.find(&identity, &Version(1, 0)).bounds(),
            Some(((&Version(1, 0), &"1.0"), (&Version(1, 5), &"1.5"),))
        );

        assert_eq!(
            impls.find(&identity, &Version(2, 1)).collect::<Vec<_>>(),
            vec![(&Version(2, 99), &"2.99"),]
        );
    }

    #[test]
    fn it_ignores_unrelated_specs() {
        let identity = "https://spec.example.com/specA";
        let unrelated = "https://spec.example.com/B";
        let impls = Implementations::new()
            .provide(identity, Version(0, 9), "too small")
            .provide(identity, Version(2, 99), "2.99")
            .provide(unrelated, Version(1, 3), "unrelated 1.3")
            .provide(identity, Version(1, 0), "1.0")
            .provide(unrelated, Version(1, 2), "unrelated 1.2")
            .provide(identity, Version(1, 2), "1.2")
            .provide(unrelated, Version(1, 5), "unrelated 1.5")
            .provide(identity, Version(1, 3), "1.3")
            .provide(identity, Version(1, 5), "1.5")
            .provide(unrelated, Version(2, 0), "2.0")
            .provide(identity, Version(2, 0), "2.0");
        assert_eq!(
            impls.find(&identity, &Version(1, 0)).collect::<Vec<_>>(),
            vec![
                (&Version(1, 0), &"1.0"),
                (&Version(1, 2), &"1.2"),
                (&Version(1, 3), &"1.3"),
                (&Version(1, 5), &"1.5"),
            ]
        );
        assert_eq!(
            impls.find(&identity, &Version(2, 1)).next(),
            Some((&Version(2, 99), &"2.99"))
        );
    }

    #[test]
    fn it_treats_each_zerodot_version_as_mutually_incompatible() {
        let identity = "https://spec.example.com/specA";
        let impls = Implementations::new()
            .provide(identity, Version(0, 0), "0.0")
            .provide(identity, Version(0, 1), "0.1")
            .provide(identity, Version(0, 2), "0.0")
            .provide(identity, Version(0, 3), "0.1")
            .provide(identity, Version(0, 99), "0.99");
        assert_eq!(
            impls.find(&identity, &Version(0, 1)).bounds(),
            Some(((&Version(0, 1), &"0.1"), (&Version(0, 1), &"0.1")))
        );
        assert_eq!(
            impls.find(&identity, &Version(0, 99)).bounds(),
            Some(((&Version(0, 99), &"0.99"), (&Version(0, 99), &"0.99")))
        );
    }
}
