use std::collections::HashSet;
use std::hash::Hash;

pub fn diff<'a, K, B, U>(basic: B, updated: U) -> (Vec<K>, Vec<K>)
where
    K: Hash + Eq + Clone + 'a,
    B: IntoIterator<Item = &'a K>,
    U: IntoIterator<Item = &'a K>,
{
    let basic: HashSet<_> = basic.into_iter().collect();
    let updated: HashSet<_> = updated.into_iter().collect();
    let to_add: Vec<_> = updated.difference(&basic).map(|k| (*k).clone()).collect();
    let to_remove: Vec<_> = basic.difference(&updated).map(|k| (*k).clone()).collect();
    (to_add, to_remove)
}

pub fn diff_full<'a, K, B, U>(basic: B, updated: U) -> (Vec<K>, Vec<K>, Vec<K>)
where
    K: Hash + Eq + Clone + 'a,
    B: IntoIterator<Item = &'a K>,
    U: IntoIterator<Item = &'a K>,
{
    let basic: HashSet<_> = basic.into_iter().collect();
    let updated: HashSet<_> = updated.into_iter().collect();
    let to_add: Vec<_> = updated.difference(&basic).map(|k| (*k).clone()).collect();
    let to_remove: Vec<_> = basic.difference(&updated).map(|k| (*k).clone()).collect();
    let to_check: Vec<_> = basic.intersection(&updated).map(|k| (*k).clone()).collect();
    (to_add, to_remove, to_check)
}
