// Standard
use std::{
    borrow::Borrow,
    fmt::Debug,
    hash::{BuildHasher, Hash, RandomState},
    iter::FusedIterator,
    ops::Index,
};

/* Required Hashmap functions
 * -- Associated --
 * [x] new
 * [x] with_capacity
 * [x] with_capacity_and_hasher
 * [x] with_hasher
 *
 * -- Methods --
 * [x] capacity                     (this returns the minimum capacity of the two)
 * [x] clear
 * [x] contains_key
 * [x] drain
 * [x] entry
 * [ ] extract_if
 * [x] get
 * [ ] get_disjoint_mut
 * [ ] get_disjoint_unchecked_mut
 * [ ] get_key_value
 * [ ] get_mut
 * [x] hasher
 * [x] insert
 * [ ] into_keys
 * [ ] into_values
 * [x] is_empty
 * [ ] iter
 * [ ] iter_mut
 * [x] keys
 * [x] len
 * [x] remove
 * [ ] remove_entry
 * [ ] reserve
 * [ ] retain
 * [ ] shrink_to
 * [ ] shrink_to_fit
 * [ ] try_reserve
 * [ ] values
 * [ ] values_mut
 */

const EMPTY_ENTRY: usize = usize::MAX;
const TOMBSTONE: usize = EMPTY_ENTRY - 1;
const MAX_LOAD_FACTOR: f64 = 0.75;
const MIN_CAPACITY: usize = 16;

// OrderedHashMap ---------------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct IEntry<K, V> {
    hash: u64,
    key: K,
    value: V,
}

#[derive(Debug, Clone, Copy)]
struct ITableIndex(usize);

#[derive(Debug, Clone)]
pub struct OrderedHashMap<K, V, S = RandomState> {
    hasher: S,
    entries: Vec<IEntry<K, V>>,
    itable: Vec<usize>,
    n_graves: usize,
}

impl<K, V, S> OrderedHashMap<K, V, S>
where
    S: BuildHasher,
    K: Hash + Eq,
{
    // Associated functions -----------------------------------------------------------------------
    pub fn with_hasher(s: S) -> Self {
        Self {
            hasher: s,
            entries: Vec::new(),
            itable: Vec::new(),
            n_graves: 0,
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        let mut out = Self {
            hasher,
            entries: Vec::with_capacity(capacity),
            itable: Vec::with_capacity(
                ((MAX_LOAD_FACTOR * capacity as f64) as usize).next_power_of_two(),
            ),
            n_graves: 0,
        };

        out.itable.fill(EMPTY_ENTRY);
        out
    }

    // OrderedHashMap Methods ---------------------------------------------------------------------
    fn iinsert(&mut self, key: K, value: V) -> Option<&mut IEntry<K, V>> {
        self.resize();

        let i_itable = self.ilookup_from_key(&key);
        match self.itable[i_itable.0] {
            EMPTY_ENTRY => {
                let hash = self.hasher.hash_one(&key);
                self.entries.push(IEntry { hash, key, value });
                self.itable[i_itable.0] = self.entries.len() - 1;
                None
            }
            i_entry => {
                let _ = std::mem::replace(&mut self.entries[i_entry].value, value);
                Some(&mut self.entries[i_entry])
            }
        }
    }

    fn ilookup<Q>(&self, hash: u64, key: &Q) -> ITableIndex
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mask = self.itable.len() - 1;
        let mut i_itable = hash as usize & mask;
        let mut i = 0;
        loop {
            let index = ITableIndex(i_itable as usize);
            match self.itable[i_itable] {
                TOMBSTONE => {}
                EMPTY_ENTRY => return index,
                _ => {
                    let entry = &self.entries[self.itable[i_itable]];
                    if entry.hash == hash && entry.key.borrow() == key {
                        return index;
                    }
                }
            };
            i += 1;
            i_itable = (i_itable + i) & mask;
        }
    }

    fn ilookup_from_key<Q>(&self, key: &Q) -> ITableIndex
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.ilookup(self.hasher.hash_one(key), key)
    }

    fn iremove<Q>(&mut self, hash: u64, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        // Check if entry exists
        if self.itable.is_empty() {
            return None;
        }
        let i_itable = self.ilookup(hash, key);

        if self.is_slot_empty(i_itable) {
            return None;
        }

        // Remove
        let i_entries = self.itable[i_itable.0];
        let out = self.entries.remove(i_entries);
        self.itable[i_itable.0] = TOMBSTONE;
        self.n_graves += 1;

        // Propagate changes
        for i in &mut self.itable {
            if ((i_entries + 1)..TOMBSTONE).contains(i) {
                *i -= 1;
            }
        }
        Some((out.key, out.value))
    }

    fn iremove_from_key<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.iremove(self.hasher.hash_one(key), key)
    }

    fn is_slot_empty(&self, i_itable: ITableIndex) -> bool {
        self.itable[i_itable.0] == EMPTY_ENTRY
    }

    #[allow(unused)]
    fn is_slot_tombstone(&self, i_itable: ITableIndex) -> bool {
        self.itable[i_itable.0] == TOMBSTONE
    }

    fn load_factor(&self) -> f64 {
        if self.occupied() == 0 {
            return 0f64;
        }
        self.occupied() as f64 / self.itable.len() as f64
    }

    fn occupied(&self) -> usize {
        self.n_graves + self.entries.len()
    }

    fn rehash(&mut self) {
        if self.itable.is_empty() {
            return;
        }
        self.itable.fill(EMPTY_ENTRY);
        self.n_graves = 0;

        for (i, entry) in self.entries.iter().enumerate() {
            let i_itable = self.ilookup(entry.hash, &entry.key);
            self.itable[i_itable.0] = i;
        }
    }

    fn resize(&mut self) {
        if self.itable.is_empty() {
            self.itable.resize(MIN_CAPACITY, EMPTY_ENTRY);
            return;
        }

        if self.load_factor() < MAX_LOAD_FACTOR {
            return;
        }

        let size = self.itable.len();
        self.itable.resize(size * 2, EMPTY_ENTRY);
        self.rehash()
    }

    pub fn last_entry(&mut self) -> OccupiedEntry<'_, K, V, S> {
        let mut max = 0usize;
        for i in &self.itable {
            if *i != EMPTY_ENTRY {
                max = max.max(*i);
            }
        }

        OccupiedEntry {
            i: ITableIndex(max),
            map: self,
        }
    }

    // HashMap Methods ----------------------------------------------------------------------------
    pub fn capacity(&self) -> usize {
        self.entries
            .capacity()
            .min((MAX_LOAD_FACTOR * self.itable.len() as f64) as usize - self.n_graves)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.itable.clear();
        self.n_graves = 0;
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.itable.is_empty() {
            return false;
        }
        let i_itable = self.ilookup_from_key(k);
        match self.itable[i_itable.0] {
            EMPTY_ENTRY => false,
            _ => true,
        }
    }

    pub fn drain<T>(&mut self) -> impl DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        self.itable.clear();
        self.n_graves = 0;
        self.entries.drain(..).map(|e| (e.key, e.value))
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S> {
        if self.itable.is_empty() {
            let hash = self.hasher.hash_one(&key);
            return Entry::Vacant(VacantEntry {
                key,
                hash,
                map: self,
            });
        }

        let i_itable = self.ilookup_from_key(&key);
        match self.itable[i_itable.0] {
            EMPTY_ENTRY => {
                let hash = self.hasher.hash_one(&key);
                Entry::Vacant(VacantEntry {
                    key,
                    hash,
                    map: self,
                })
            }
            _ => Entry::Occupied(OccupiedEntry {
                i: i_itable,
                map: self,
            }),
        }
    }

    // pub fn extract_if

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if self.itable.is_empty() {
            return None;
        }
        let i_itable = self.ilookup_from_key(k);
        match self.itable[i_itable.0] {
            EMPTY_ENTRY => None,
            i_entry => Some(&self.entries[i_entry].value),
        }
    }

    pub fn hasher(&self) -> &S {
        return &self.hasher;
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.resize();

        let i_itable = self.ilookup_from_key(&key);
        match self.itable[i_itable.0] {
            EMPTY_ENTRY => {
                let hash = self.hasher.hash_one(&key);
                self.entries.push(IEntry { hash, key, value });
                self.itable[i_itable.0] = self.entries.len() - 1;
                None
            }
            i_entry => Some(std::mem::replace(&mut self.entries[i_entry].value, value)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.entries.iter(),
        }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.entries.iter(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.iremove_from_key(k).map(|(_, v)| v)
    }
}

impl<K, V, S> FromIterator<(K, V)> for OrderedHashMap<K, V, S>
where
    S: Default + BuildHasher,
    K: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut out: OrderedHashMap<K, V, S> = OrderedHashMap::with_hasher(S::default());
        for (k, v) in iter.into_iter() {
            out.insert(k, v);
        }
        out
    }
}

impl<K, V, S> Default for OrderedHashMap<K, V, S>
where
    S: Default + BuildHasher,
    K: Hash + Eq,
{
    fn default() -> Self {
        OrderedHashMap::with_hasher(S::default())
    }
}

impl<K, V> OrderedHashMap<K, V, RandomState>
where
    K: Hash + Eq,
{
    // Associated functions -----------------------------------------------------------------------
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        OrderedHashMap::with_capacity_and_hasher(capacity, Default::default())
    }
}

impl<Q, K, V, S> Index<&Q> for OrderedHashMap<K, V, S>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq,
    S: BuildHasher,
{
    type Output = V;
    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("key not found")
    }
}

// Entry ------------------------------------------------------------------------------------------
pub enum Entry<'a, K, V, S> {
    Occupied(OccupiedEntry<'a, K, V, S>),
    Vacant(VacantEntry<'a, K, V, S>),
}

impl<'a, K, V, S> Entry<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut e) => {
                f(e.get_mut());
                Entry::Occupied(e)
            }
            Entry::Vacant(_) => self,
        }
    }

    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, S> {
        match self {
            Entry::Occupied(mut e) => {
                e.insert(value);
                e
            }
            Entry::Vacant(e) => e.insert_entry(value),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(e) => e.key(),
            Entry::Vacant(e) => &e.key,
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(default),
        }
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(default()),
        }
    }

    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let v = default(&e.key);
                e.insert(v)
            }
        }
    }
}

impl<'a, K, V, S> Entry<'a, K, V, S>
where
    K: Hash + Eq,
    V: Default,
    S: BuildHasher,
{
    pub fn or_default(self) -> &'a V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(Default::default()),
        }
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V, S> {
    i: ITableIndex,
    map: &'a mut OrderedHashMap<K, V, S>,
}

impl<'a, K, V, S> OccupiedEntry<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    // OccupiedEntry - Private -------------------------------------------------------------------
    fn get_entry(&self) -> &IEntry<K, V> {
        let i_entry = self.map.itable[self.i.0];
        &self.map.entries[i_entry]
    }

    fn get_entry_mut(&mut self) -> &mut IEntry<K, V> {
        let i_entry = self.map.itable[self.i.0];
        &mut self.map.entries[i_entry]
    }
    // OccupiedEntry - Public ---------------------------------------------------------------------
    pub fn get(&self) -> &V {
        &self.get_entry().value
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.get_entry_mut().value
    }

    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }

    pub fn into_mut(self) -> &'a mut V {
        let i_entry = self.map.itable[self.i.0];
        &mut self.map.entries[i_entry].value
    }

    pub fn key(&self) -> &K {
        &self.get_entry().key
    }

    pub fn remove(self) -> V {
        todo!()
    }

    pub fn remove_entry(self) -> (K, V) {
        todo!()
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct VacantEntry<'a, K, V, S> {
    key: K,
    hash: u64,
    map: &'a mut OrderedHashMap<K, V, S>,
}

impl<'a, K, V, S> VacantEntry<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    pub fn insert(self, value: V) -> &'a mut V {
        &mut self
            .map
            .iinsert(self.key, value)
            .expect("invariant broken")
            .value
    }

    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, S> {
        self.map.insert(self.key, value);
        self.map.last_entry()
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

// Keys -------------------------------------------------------------------------------------------
pub struct Keys<'a, K, V> {
    iter: std::slice::Iter<'a, IEntry<K, V>>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| &e.key)
    }
}

// Iter -------------------------------------------------------------------------------------------
pub struct Iter<'a, K, V> {
    iter: std::slice::Iter<'a, IEntry<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| (&e.key, &e.value))
    }
}

// Unit Testing -----------------------------------------------------------------------------------
#[cfg(test)]
mod ohm_tests {
    use super::*;
    use std::{
        collections::HashMap,
        fmt::Debug,
        hash::{BuildHasherDefault, Hasher},
    };

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut state = self.0;
            state ^= state << 7;
            state ^= state >> 9;
            self.0 = state;
            self.0
        }
        fn within(&mut self, lim: u64) -> u64 {
            self.next() & lim
        }
    }

    #[derive(Default)]
    struct BadHasher(u64);
    impl Hasher for BadHasher {
        fn finish(&self) -> u64 {
            self.0 % 4
        }
        fn write(&mut self, bytes: &[u8]) {
            for b in bytes {
                self.0 = self.0.wrapping_mul(31).wrapping_add(*b as u64);
            }
        }
    }
    type BadBuild = BuildHasherDefault<BadHasher>;

    type Reference = HashMap<u32, u32, BadBuild>;
    type Target = OrderedHashMap<u32, u32, BadBuild>;

    fn assert_state(reference: &Reference, target: &Target) {
        assert_eq!(reference.len(), target.len(), "missing/extra entries");
        assert_eq!(
            reference.hasher().hash_one(0xABCD),
            target.hasher().hash_one(0xABCD),
            "hasher de-sync"
        );

        for key in reference.keys() {
            assert_eq!(
                reference.get(key),
                target.get(key),
                "\nkey: {}\nref: {:?}\ntarget_itable: {:?}\ntarget_entries: {:#?}",
                &key,
                reference.iter().collect::<Vec<_>>(),
                &target.itable,
                &target.entries,
            )
        }
    }

    #[test]
    fn test_against_ref() {
        let mut rng = Rng(0xABCD);
        let mut keys: Vec<u32> = Vec::new();
        let mut reference: HashMap<u32, u32, BadBuild> = HashMap::with_hasher(BadBuild::new());
        let mut target: OrderedHashMap<u32, u32, BadBuild> =
            OrderedHashMap::with_hasher(BadBuild::new());
        assert_state(&reference, &target);

        for _ in 0..100_000 {
            match rng.within(3) {
                2 => {
                    let (k, v) = (rng.next() as u32, rng.next() as u32);
                    println!("Inserting {}, {}", &k, &v);
                    assert_eq!(reference.insert(k, v), target.insert(k, v));
                    keys.push(k);
                }
                1 => {
                    let k = keys.pop();
                    print!("Removing ");
                    match k {
                        None => {
                            println!(" nothing");
                            assert_eq!(reference.len(), target.len())
                        }
                        Some(k) => {
                            println!("{}", &k);
                            assert_eq!(reference.remove(&k), target.remove(&k))
                        }
                    }
                }
                0 => {
                    println!("Clearing");
                    reference.clear();
                    target.clear();
                    assert_eq!(reference.len(), target.len());
                }
                _ => {}
            }
            assert_state(&reference, &target);
        }
    }

    fn val_entries_and_order<K, V, S, const N: usize>(
        ohm: &OrderedHashMap<K, V, S>,
        entries: [(K, V); N],
    ) where
        K: Debug + Eq,
        V: Debug + Eq,
        S: BuildHasher,
    {
        for (s, t) in ohm.entries.iter().zip(entries) {
            assert_eq!(s.key, t.0, "key mismatch");
            assert_eq!(s.value, t.1, "key mismatch");
        }
    }

    #[test]
    fn test_make() {
        let _: OrderedHashMap<i32, i32> = OrderedHashMap::new();
    }

    #[test]
    fn test_make_with_capacity() {
        let _: OrderedHashMap<i32, i32> = OrderedHashMap::with_capacity(100);
    }

    #[test]
    fn test_insert_no_capacity() {
        let mut ohm: OrderedHashMap<i32, i32> = OrderedHashMap::new();

        ohm.insert(1, 2);
        ohm.insert(3, 4);
        ohm.insert(5, 6);

        val_entries_and_order(&ohm, [(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn test_insert_with_capacity() {
        let mut ohm: OrderedHashMap<i32, i32> = OrderedHashMap::with_capacity(100);

        ohm.insert(1, 2);
        ohm.insert(3, 4);
        ohm.insert(5, 6);

        val_entries_and_order(&ohm, [(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn test_index_empty() {
        let ohm: OrderedHashMap<i32, i32> = OrderedHashMap::with_capacity(100);
        assert_eq!(ohm.get(&1000), None);
    }

    #[test]
    fn test_remove() {
        let mut ohm: OrderedHashMap<i32, i32> = OrderedHashMap::with_capacity(100);

        ohm.insert(1, 2);
        ohm.insert(3, 4);
        ohm.insert(5, 6);

        ohm.remove(&3);

        val_entries_and_order(&ohm, [(1, 2), (5, 6)]);
    }

    #[test]
    fn test_realloc() {
        const N: usize = 1000;
        let mut ohm: OrderedHashMap<usize, usize, BadBuild> =
            OrderedHashMap::with_hasher(BadBuild::new());
        for i in 0..N {
            // print!("iter {i}, lf={}: ", ohm.load_factor());
            ohm.insert(i, i);
            // println!("done");
        }

        println!(
            "sizeof<[(usize, usize); {}]>: {}MB",
            N,
            N * std::mem::size_of::<[(usize, usize); N]>() / 1000usize.pow(2)
        );

        val_entries_and_order(&ohm, std::array::from_fn::<_, N, _>(|i| (i, i)));
    }
}
