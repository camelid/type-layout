#![allow(dead_code, unused_macros)] // TODO: remove dead code

pub mod list;

use std::{convert::TryFrom, fmt::Display, ops::RangeInclusive};

// MAPS AND SETS //

pub(crate) type Map<K, V> = std::collections::BTreeMap<K, V>;

macro_rules! map {
    { $($key:expr => $value:expr),* $(,)? } => {
        {
            #[allow(unused_mut)]
            let mut map = $crate::util::Map::new();
            $(map.insert($key.into(), $value);)*
            map
        }
    };
}

pub(crate) type Set<T> = std::collections::BTreeSet<T>;

macro_rules! set {
    { $($value:expr),* $(,)? } => {
        {
            #[allow(unused_mut)]
            let mut set = $crate::util::Set::new();
            $(set.insert($value);)*
            set
        }
    };
}

// UTILITY FUNCTIONS //

pub(crate) fn b<T>(x: T) -> Box<T> {
    Box::new(x)
}

#[track_caller]
pub(crate) fn expect_singleton_vec<T>(v: Vec<T>) -> T {
    let actual_len = v.len();
    match <[T; 1]>::try_from(v) {
        Ok([one]) => one,
        Err(_) => panic!("expected singleton Vec, but has {} elements", actual_len),
    }
}

pub(crate) fn result_map_both<T, U, F: FnOnce(T) -> U>(f: F, r: Result<T, T>) -> Result<U, U> {
    match r {
        Ok(ok) => Ok(f(ok)),
        Err(err) => Err(f(err)),
    }
}

// DISPLAY //

pub fn display_map<K: Display, V: Display>(m: impl ExactSizeIterator<Item = (K, V)>) -> String {
    display_map_like(m, " => ", ", ")
}

pub fn display_map_like<K: Display, V: Display>(
    m: impl ExactSizeIterator<Item = (K, V)>,
    kv_sep: &str,
    entry_sep: &str,
) -> String {
    if m.len() == 0 {
        String::from("{}")
    } else {
        format!(
            "{{ {} }}",
            m.map(|(n, l)| format!("{}{}{}", n, kv_sep, l))
                .intersperse(String::from(entry_sep))
                .collect::<String>()
        )
    }
}

// RANGES ///

pub fn range_values_count(r: RangeInclusive<u64>) -> Option<u64> {
    if r.is_empty() {
        Some(0)
    } else {
        r.end().checked_sub(*r.start()).and_then(|x| x.checked_add(1))
    }
}

#[cfg(test)]
mod tests;
