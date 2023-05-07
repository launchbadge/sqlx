/// Simplistic map implementation built on a Vec of Options (index = key)
#[derive(Debug, Clone, Eq, Default)]
pub(crate) struct IntMap<V: std::fmt::Debug + Clone + Eq + PartialEq + std::hash::Hash>(
    Vec<Option<V>>,
);

impl<V: std::fmt::Debug + Clone + Eq + PartialEq + std::hash::Hash> IntMap<V> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn expand(&mut self, size: i64) -> usize {
        let idx = size.try_into().expect("negative column index unsupported");
        while self.0.len() <= idx {
            self.0.push(None);
        }
        idx
    }

    pub(crate) fn from_dense_record(record: &Vec<V>) -> Self {
        Self(record.iter().cloned().map(Some).collect())
    }

    pub(crate) fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.iter_mut().filter_map(Option::as_mut)
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().filter_map(Option::as_ref)
    }

    pub(crate) fn get(&self, idx: &i64) -> Option<&V> {
        let idx: usize = (*idx)
            .try_into()
            .expect("negative column index unsupported");

        match self.0.get(idx) {
            Some(Some(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn get_mut(&mut self, idx: &i64) -> Option<&mut V> {
        let idx: usize = (*idx)
            .try_into()
            .expect("negative column index unsupported");
        match self.0.get_mut(idx) {
            Some(Some(v)) => Some(v),
            _ => None,
        }
    }

    pub(crate) fn insert(&mut self, idx: i64, value: V) -> Option<V> {
        let idx: usize = self.expand(idx);

        std::mem::replace(&mut self.0[idx], Some(value))
    }

    pub(crate) fn remove(&mut self, idx: &i64) -> Option<V> {
        let idx: usize = (*idx)
            .try_into()
            .expect("negative column index unsupported");

        let item = self.0.get_mut(idx);
        match item {
            Some(content) => std::mem::replace(content, None),
            None => None,
        }
    }
}

impl<V: std::fmt::Debug + Clone + Eq + PartialEq + std::hash::Hash> std::hash::Hash for IntMap<V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for value in self.values() {
            value.hash(state);
        }
    }
}

impl<V: std::fmt::Debug + Clone + Eq + PartialEq + std::hash::Hash> PartialEq for IntMap<V> {
    fn eq(&self, other: &Self) -> bool {
        if !self
            .0
            .iter()
            .zip(other.0.iter())
            .all(|(l, r)| PartialEq::eq(l, r))
        {
            return false;
        }

        if self.0.len() > other.0.len() {
            self.0[other.0.len()..].iter().all(Option::is_none)
        } else if self.0.len() < other.0.len() {
            other.0[self.0.len()..].iter().all(Option::is_none)
        } else {
            true
        }
    }
}

impl<V: std::fmt::Debug + Clone + Eq + PartialEq + std::hash::Hash + Default> FromIterator<(i64, V)>
    for IntMap<V>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (i64, V)>,
    {
        let mut result = Self(Vec::new());
        for (idx, val) in iter {
            let idx = result.expand(idx);
            result.0[idx] = Some(val);
        }
        result
    }
}
