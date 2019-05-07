use std::collections::BTreeMap;

pub trait Merge {
    /// Merge another instance into this one.
    ///
    /// Values defined in `other` take precedence over values defined in `self`.
    fn merge(self, other: Self) -> Self;
}

impl<T> Merge for Option<T> {
    #[inline]
    fn merge(self, other: Self) -> Self {
        other.or(self)
    }
}

impl<K: std::hash::Hash + Ord, V: Merge> Merge for BTreeMap<K, V> {
    fn merge(self, other: Self) -> Self {
        let mut merged = self;
        for (k, v) in other.into_iter() {
            let vmerged = if let Some(vself) = merged.remove(&k) {
                vself.merge(v)
            } else {
                v
            };
            merged.insert(k, vmerged);
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use crate::Merge;
    use std::collections::BTreeMap;

    #[test]
    fn option() {
        let a = Option::Some(1);
        let b = Option::Some(2);
        let none = Option::None;

        assert_eq!(a.merge(b), b);
        assert_eq!(a.merge(none), a);
        assert_eq!(none.merge(b), b);
        assert_eq!(none.merge(none), none);
    }

    #[test]
    fn btree_map() {
        let mut a = BTreeMap::new();
        a.insert("a", Some("a-value"));
        a.insert("b", Some("a-value"));
        a.insert("c", None);

        let mut b = BTreeMap::new();
        b.insert("a", Some("b-value"));
        b.insert("b", None);
        b.insert("c", Some("b-value"));
        b.insert("d", Some("b-value"));

        let merged = a.merge(b);
        let mut expected = BTreeMap::new();
        expected.insert("a", Some("b-value"));
        expected.insert("b", Some("a-value"));
        expected.insert("c", Some("b-value"));
        expected.insert("d", Some("b-value"));
        assert_eq!(merged, expected);
    }
}
