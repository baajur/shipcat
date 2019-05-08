use std::collections::BTreeMap;
use shipcat_definitions::{Result};
use super::RelaxedString;

pub trait Build<T, P> {
    fn build(self, params: &P) -> Result<T>;
}

impl<T, P, S: Build<T, P>> Build<Option<T>, P> for Option<S> {
    fn build(self, params: &P) -> Result<Option<T>> {
        self.map(|s| s.build(params)).transpose()
    }
}

impl<T, P, S: Build<T, P>> Build<Vec<T>, P> for Vec<S> {
    fn build(self, params: &P) -> Result<Vec<T>> {
        self.into_iter()
            .map(|s| s.build(params))
            .collect()
    }
}

impl<K, V, P, S> Build<BTreeMap<K, V>, P> for BTreeMap<K, Option<S>> where
    K: std::hash::Hash + Ord,
    S: Build<V, P>,
{
    fn build(self, params: &P) -> Result<BTreeMap<K, V>> {
        self.into_iter()
            .filter_map(|(k, maybe)| maybe.map(|s| (k, s)))
            .map(|(k, s)| s.build(params).map(|v| (k, v)))
            .collect()
    }
}

impl Build<String, ()> for RelaxedString {
    fn build(self, _: &()) -> Result<String> {
        Ok(self.to_string())
    }
}

impl Build<String, ()> for String {
    fn build(self, _: &()) -> Result<Self> {
        Ok(self)
    }
}
