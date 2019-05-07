use std::collections::BTreeMap;
use merge::Merge;

use shipcat_definitions::Result;

use super::Build;

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum NamedList<T> {
    MapBacked(BTreeMap<String, EnabledWrapper<T>>),
    ListBacked(Vec<NameWrapper<T>>),
}

use self::NamedList::{ListBacked, MapBacked};

pub struct NameParams<T> {
    pub name: String,
    pub params: T,
}

impl<B, S, P> Build<Vec<B>, P> for NamedList<S> where
    P: Clone,
    S: Build<B, NameParams<P>>,
{
    fn build(self, params: &P) -> Result<Vec<B>> {
        let entries: BTreeMap<String, S> = self.into();
        let mut items = Vec::new();
        for (k, v) in entries {
            let item = v.build(&NameParams {
                name: k,
                params: params.clone(),
            })?;
            items.push(item);
        }
        Ok(items)
    }
}

impl<T> Into<BTreeMap<String, T>> for NamedList<T> {
    fn into(self) -> BTreeMap<String, T> {
        let mut entries = BTreeMap::new();
        match self {
            MapBacked(vs) => {
                for (k, v) in vs {
                    if v.enabled.unwrap_or(true) {
                        entries.insert(k, v.item);
                    }
                }
            },
            ListBacked(xs) => {
                for NameWrapper { name, item } in xs {
                    entries.insert(name, item);
                }
            }
        }
        entries
    }
}

impl<T> Into<BTreeMap<String, EnabledWrapper<T>>> for NamedList<T> {
    fn into(self) -> BTreeMap<String, EnabledWrapper<T>> {
        match self {
            MapBacked(x) => x,
            ListBacked(xs) => {
                let mut entries = BTreeMap::new();
                for NameWrapper { name, item } in xs {
                    entries.insert(name, EnabledWrapper { enabled: Some(true), item });
                }
                entries
            }
        }
    }
}

impl<T: Merge> Merge for NamedList<T> {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (s @ _, MapBacked(o)) => {
                let s: BTreeMap<String, EnabledWrapper<T>> = s.into();
                let o = o.into();
                MapBacked(s.merge(o))
            }
            (_, other @ ListBacked(_)) => other,
        }
    }
}

#[derive(Clone, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct EnabledWrapper<T> {
    pub enabled: Option<bool>,

    #[serde(flatten)]
    pub item: T,
}

impl<T: Merge> Merge for EnabledWrapper<T> {
    fn merge(self, other: Self) -> Self {
        Self {
            enabled: self.enabled.merge(other.enabled),
            item: self.item.merge(other.item),
        }
    }
}

#[derive(Clone, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct NameWrapper<T> {
    pub name: String,

    #[serde(flatten)]
    pub item: T,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use merge::Merge;
    use shipcat_definitions::Result;

    use crate::util::Build;
    use super::{NameWrapper, EnabledWrapper, NameParams};
    use super::NamedList::{ListBacked,MapBacked};

    #[derive(Clone, Debug, PartialEq, Merge)]
    pub struct ExampleSource {
        value: Option<u32>,
    }

    impl ExampleSource {
        fn new(value: u32) -> Self {
            ExampleSource {
                value: Some(value),
            }
        }
    }

    impl Build<String, NameParams<String>> for ExampleSource {
        fn build(self, params: &NameParams<String>) -> Result<String> {
            Ok(if let Some(v) = self.value {
                format!("{}{}{}", params.name, params.params, v)
            } else {
                params.name.to_string()
            })
        }
    }

    #[test]
    fn merge() {
        let x_list = ListBacked(vec!(
            NameWrapper { name: "foo".into(), item: ExampleSource::new(0) },
            NameWrapper { name: "bar".into(), item: ExampleSource { value: None } },
        ));
        assert_eq!(MapBacked(x_list.clone().into()), MapBacked(btreemap!{
            "foo".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource::new(0) },
            "bar".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource { value: None } },
        }));
        let x_map = MapBacked(btreemap!{
            "foo".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(0) },
            "bar".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource { value: None } },
            "blort".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource::new(2) },
        });

        // Merging from a list always returns the list
        let empty_list = ListBacked(Vec::new());
        assert_eq!(x_list.clone().merge(empty_list.clone()), empty_list);
        assert_eq!(x_map.clone().merge(empty_list.clone()), empty_list);
        let y_list = ListBacked(vec!(
            NameWrapper { name: "blort".into(), item: ExampleSource::new(0) },
        ));
        assert_eq!(x_list.clone().merge(y_list.clone()), y_list);
        assert_eq!(x_map.clone().merge(y_list.clone()), y_list);

        // Merging map-backed merges entries
        let empty_map = MapBacked(BTreeMap::new());
        assert_eq!(x_list.clone().merge(empty_map.clone()), MapBacked(x_list.clone().into()));
        assert_eq!(x_map.clone().merge(empty_map.clone()), x_map);

        let y_map = MapBacked(btreemap!{
            "foo".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource { value: None } },
            "bar".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(1000) },
            "foobar".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(1001) },
        });
        assert_eq!(x_list.clone().merge(y_map.clone()), MapBacked(btreemap!{
            "foo".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource::new(0) },
            "bar".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource::new(1000) },
            "foobar".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(1001) },
        }));
        assert_eq!(x_map.clone().merge(y_map.clone()), MapBacked(btreemap!{
            "foo".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource::new(0) },
            "bar".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource::new(1000) },
            "blort".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource::new(2) },
            "foobar".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(1001) },
        }));
    }

    #[test]
    fn build() {
        let params = ":".to_string();

        let x_list = ListBacked(vec!(
            NameWrapper { name: "foo".into(), item: ExampleSource::new(0) },
            NameWrapper { name: "bar".into(), item: ExampleSource { value: None } },
        ));
        let mut actual = x_list.build(&params).unwrap();
        actual.sort();
        assert_eq!(actual, vec!("bar", "foo:0"));

        let x_map = MapBacked(btreemap!{
            // Included
            "foo".into() => EnabledWrapper { enabled: None, item: ExampleSource::new(0) },
            "bar".into() => EnabledWrapper { enabled: Some(true), item: ExampleSource { value: None } },
            // Ignored
            "blort".into() => EnabledWrapper { enabled: Some(false), item: ExampleSource::new(2) },
        });
        let mut actual = x_map.build(&params).unwrap();
        actual.sort();
        assert_eq!(actual, vec!("bar", "foo:0"));
    }
}
