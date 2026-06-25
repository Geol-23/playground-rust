// Standard
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::IntoIterator;
use std::vec;


pub struct OrderedHashMap<K, V> {
    map: HashMap<K, V>,
    order: Vec<K>,
}

impl <K, V> OrderedHashMap<K, V>
where
    K: Eq + Hash + Clone
{
    pub fn new() -> Self {
        OrderedHashMap {
            map: HashMap::new(),
            order: Vec::new()
        }
    }

    pub fn clear(self: &mut Self) {
        self.map.clear();
        self.order.clear();
    }

    pub fn contains_key<Q>(self: &Self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(k)
    }

    pub fn get<Q>(self: &Self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.get(k)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.map.insert(k.clone(), v) {
            Some(v) => { Some(v) },
            None => { self.order.push(k); None }
        }
    }

    pub fn remove<Q>(self: &mut Self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let v = self.map.remove(k)?;
        self.order.iter()
            .position(|key| key.borrow() == k)
            .map(|i| {
                self.order.remove(i);
                v
            })
    }
}


pub struct IntoIter<K, V> {
    map: HashMap<K, V>,
    iter: vec::IntoIter<K>,
}

impl<K, V> IntoIter<K, V>
where
    K: Eq + Hash + Clone
{
    fn new(other: OrderedHashMap<K, V>) -> Self {
        Self {
            map: other.map,
            iter: other.order.into_iter(),
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Eq + Hash + Clone
{
    type Item = (K, V);

    fn next(self: &mut Self) -> Option<Self::Item> {
        let key = self.iter.next()?;
        let value = self.map.remove(&key)?;
        Some((key, value))
    }
}

impl<K, V> IntoIterator for OrderedHashMap<K, V>
where
    K: Eq + Hash + Clone
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
