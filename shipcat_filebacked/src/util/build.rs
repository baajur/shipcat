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
        let mut ts = Vec::new();
        for s in self {
            let t = s.build(params)?;
            ts.push(t);
        }
        Ok(ts)
    }
}

impl<K, V, P, S> Build<BTreeMap<K, V>, P> for BTreeMap<K, Option<S>> where
    K: std::hash::Hash + Ord,
    S: Build<V, P>,
{
    fn build(self, params: &P) -> Result<BTreeMap<K, V>> {
        let mut map = BTreeMap::new();
        for (k, s) in self {
            if let Some(s) = s {
                let v = s.build(params)?;
                map.insert(k, v);
            }
        }
        Ok(map)
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
