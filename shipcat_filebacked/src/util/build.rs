use shipcat_definitions::{Result};

pub trait Build<T, P> {
    fn build(self, params: &P) -> Result<T>;
}

impl<T, P, S: Build<T, P>> Build<Option<T>, P> for Option<S> {
    fn build(self, params: &P) -> Result<Option<T>> {
        match self {
            None => Ok(None),
            Some(s) => {
                let t = s.build(params)?;
                Ok(Some(t))
            }
        }
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
